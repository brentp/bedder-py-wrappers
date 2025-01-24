use pyo3::exceptions::{PyKeyError, PyRuntimeError};
use pyo3::{prelude::*, types::PyDict, IntoPyObjectExt};
use rust_htslib::bcf::header::TagType;
use rust_htslib::{bcf, bcf::Read};
use std::rc::Rc;

#[pyclass(unsendable)]
struct PyReader {
    reader: bcf::Reader,
}

#[pymethods]
impl PyReader {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let reader = bcf::Reader::from_path(path)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to open file: {}", e)))?;
        Ok(PyReader { reader })
    }

    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }

    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyRecord> {
        let mut record = slf.reader.empty_record();
        match slf.reader.read(&mut record) {
            Some(Ok(())) => Some(PyRecord { record }),
            None => None,
            Some(Err(_)) => None,
        }
    }

    fn header(&self) -> PyResult<PyHeader> {
        Ok(PyHeader {
            inner: Rc::new(self.reader.header().inner),
        })
    }
}

#[pyclass(unsendable)]
struct PyHeader {
    inner: Rc<*mut rust_htslib::htslib::bcf_hdr_t>,
}

#[pymethods]
impl PyHeader {
    fn add_info(&mut self, info: &Bound<'_, PyDict>) -> PyResult<()> {
        let id = info
            .get_item("ID")?
            .ok_or_else(|| PyRuntimeError::new_err("Missing required field 'ID'"))?
            .extract::<String>()?;
        let number = info
            .get_item("Number")?
            .ok_or_else(|| PyRuntimeError::new_err("Missing required field 'Number'"))?
            .extract::<String>()?;
        let description = info
            .get_item("Description")?
            .ok_or_else(|| PyRuntimeError::new_err("Missing required field 'Description'"))?
            .extract::<String>()?;
        let type_str = info
            .get_item("Type")?
            .ok_or_else(|| PyRuntimeError::new_err("Missing required field 'Type'"))?
            .extract::<String>()?;

        let header_line = format!(
            r#"##INFO=<ID={},Number={},Type={},Description="{}">"#,
            id, number, type_str, description
        );

        let c_str = std::ffi::CString::new(header_line.as_bytes()).unwrap();
        if 0 != unsafe { rust_htslib::htslib::bcf_hdr_append(*self.inner, c_str.as_ptr()) } {
            return Err(PyRuntimeError::new_err("Failed appending header line"));
        }
        if 0 != unsafe { rust_htslib::htslib::bcf_hdr_sync(*self.inner) } {
            return Err(PyRuntimeError::new_err("Failed to sync header"));
        }

        Ok(())
    }

    fn __repr__(slf: PyRef<'_, Self>) -> PyResult<String> {
        let mut kstr = rust_htslib::htslib::kstring_t {
            l: 0,
            m: 0,
            s: std::ptr::null_mut(),
        };
        unsafe {
            rust_htslib::htslib::bcf_hdr_format(*slf.inner, 0, &mut kstr);
        }
        let s = unsafe {
            String::from_utf8_unchecked(
                std::slice::from_raw_parts(kstr.s as *const u8, kstr.l).to_vec(),
            )
        };
        Ok(format!("PyHeader({:?})", s))
    }

    fn samples(&self) -> PyResult<Vec<String>> {
        if unsafe { (*(*self.inner)).n[2] } == 0 {
            return Ok(vec![]);
        }
        let header_view = bcf::header::HeaderView::new(*self.inner);
        let samples = header_view
            .samples()
            .iter()
            .map(|s| unsafe { String::from_utf8_unchecked(s.to_vec()) })
            .collect();
        Ok(samples)
    }

    fn has_info(&self, tag: &str) -> PyResult<bool> {
        let tag_bytes = tag.as_bytes();
        let header_view = bcf::header::HeaderView::new(*self.inner);
        Ok(header_view
            .info_type(tag_bytes)
            .map(|_| true)
            .unwrap_or(false))
    }
}

#[pyclass]
pub struct PyRecord {
    record: bcf::Record,
}

impl Drop for PyHeader {
    fn drop(&mut self) {}
}

impl PyRecord {
    /// Create a new PyRecord from a bcf::Record to allow for use in Python.
    pub fn new(record: bcf::Record) -> Self {
        Self { record }
    }
}

#[pymethods]
impl PyRecord {
    fn header(&self) -> PyResult<PyHeader> {
        Ok(PyHeader {
            inner: Rc::new(self.record.header().inner),
        })
    }

    fn __repr__(slf: PyRef<'_, Self>) -> PyResult<String> {
        let mut kstr = rust_htslib::htslib::kstring_t {
            l: 0,
            m: 0,
            s: std::ptr::null_mut(),
        };
        unsafe {
            rust_htslib::htslib::vcf_format(slf.record.header().inner, slf.record.inner, &mut kstr);
        }
        let s = unsafe {
            String::from_utf8_unchecked(
                std::slice::from_raw_parts(kstr.s as *const u8, kstr.l).to_vec(),
            )
        };
        Ok(format!("PyRecord({:?})", s))
    }

    #[getter]
    fn chrom(&self) -> PyResult<String> {
        let rid = self
            .record
            .rid()
            .ok_or(PyRuntimeError::new_err("Failed to get rid"))?;
        let chrom = self
            .record
            .header()
            .rid2name(rid)
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to get chrom: {}", e)))?;
        Ok(unsafe { String::from_utf8_unchecked(chrom.to_vec()) })
    }

    #[getter]
    fn pos(&self) -> PyResult<i32> {
        Ok(self.record.pos() as i32)
    }

    #[getter]
    fn start(&self) -> PyResult<i32> {
        Ok(self.record.pos() as i32)
    }

    #[getter]
    fn end(&self) -> PyResult<i32> {
        Ok(self.record.end() as i32)
    }

    #[getter]
    fn id(&self) -> PyResult<String> {
        let id = self.record.id();
        // now look up the ids in the header
        Ok(unsafe { String::from_utf8_unchecked(id) })
    }

    fn set_id(&mut self, id: &str) -> PyResult<()> {
        self.record
            .set_id(id.as_bytes())
            .map_err(|e| PyRuntimeError::new_err(format!("Failed to set ID: {}", e)))?;
        Ok(())
    }

    #[getter]
    fn ref_allele(&self) -> PyResult<String> {
        let ref_allele = self.record.alleles()[0];
        Ok(unsafe { String::from_utf8_unchecked(ref_allele.to_vec()) })
    }

    #[getter]
    fn alt_alleles(&self) -> PyResult<Vec<String>> {
        let alt_alleles = &self.record.alleles()[1..];
        Ok(alt_alleles
            .iter()
            .map(|allele| unsafe { String::from_utf8_unchecked(allele.to_vec()) })
            .collect())
    }

    #[getter]
    fn qual(&self) -> PyResult<f32> {
        Ok(self.record.qual())
    }

    fn set_qual(&mut self, qual: f32) -> PyResult<()> {
        self.record.set_qual(qual);
        Ok(())
    }

    #[getter]
    fn filter(&self) -> PyResult<Vec<String>> {
        let filters = self.record.filters();
        let mut filter_list = Vec::new();
        for filter in filters {
            let name = self.record.header().id_to_name(filter);
            filter_list.push(unsafe { String::from_utf8_unchecked(name.to_vec()) });
        }
        Ok(filter_list)
    }

    fn set_filter(&mut self, filters: Vec<String>) -> PyResult<()> {
        // Remove all existing filters by setting to PASS

        let current_filters = self.record.filters().collect::<Vec<_>>();
        for filter in current_filters {
            self.record
                .remove_filter(&filter, true)
                .map_err(|e| PyRuntimeError::new_err(format!("Failed to remove filter: {}", e)))?;
        }

        // Add new filters
        for filter in filters {
            let filter_bytes = filter.as_bytes();
            self.record.push_filter(filter_bytes).map_err(|e| {
                PyRuntimeError::new_err(format!("Failed to set filter '{}': {}", filter, e))
            })?;
        }
        Ok(())
    }

    fn set_info(&mut self, tag: &str, values: PyObject) -> PyResult<()> {
        let tag_bytes = tag.as_bytes();
        let info_type = self.record.header().info_type(tag_bytes).map_err(|e| {
            PyRuntimeError::new_err(format!("INFO field {} not in header: {}", tag, e))
        })?;

        Python::with_gil(|py| {
            match info_type.0 {
                TagType::Integer => {
                    let values = values.extract::<Vec<i32>>(py)?;
                    self.record
                        .push_info_integer(tag_bytes, &values)
                        .map_err(|e| {
                            PyRuntimeError::new_err(format!("Failed to set integer info: {}", e))
                        })?;
                }
                TagType::Float => {
                    let values = values.extract::<Vec<f32>>(py)?;
                    self.record
                        .push_info_float(tag_bytes, &values)
                        .map_err(|e| {
                            PyRuntimeError::new_err(format!("Failed to set float info: {}", e))
                        })?;
                }
                TagType::String => {
                    let values = values.extract::<Vec<String>>(py)?;
                    let bytes: Vec<&[u8]> = values.iter().map(|s| s.as_bytes()).collect();
                    self.record
                        .push_info_string(tag_bytes, &bytes)
                        .map_err(|e| {
                            PyRuntimeError::new_err(format!("Failed to set string info: {}", e))
                        })?;
                }
                _ => return Err(PyRuntimeError::new_err("Unsupported INFO type")),
            }
            Ok(())
        })
    }

    fn info(&self, tag: &str) -> PyResult<PyObject> {
        let tag_bytes = tag.as_bytes();
        let info = self.record.info(tag_bytes);
        let info_type = self
            .record
            .header()
            .info_type(tag_bytes)
            .map_err(|e| PyRuntimeError::new_err(format!("INFO access failed: {}", e)))?;

        let tag_type = info_type.0;

        Python::with_gil(|py| match tag_type {
            TagType::Integer => {
                let values = info
                    .integer()
                    .map_err(|_| PyRuntimeError::new_err("Failed to get integer values"))?;
                if let Some(values) = values {
                    values.iter().copied().collect::<Vec<_>>().into_py_any(py)
                } else {
                    let empty: Vec<i32> = vec![];
                    empty.into_py_any(py)
                }
            }
            TagType::Float => {
                let values = info
                    .float()
                    .map_err(|_| PyRuntimeError::new_err("Failed to get float values"))?;
                if let Some(values) = values {
                    values.iter().copied().collect::<Vec<_>>().into_py_any(py)
                } else {
                    let empty: Vec<f32> = vec![];
                    empty.into_py_any(py)
                }
            }
            TagType::String => {
                let values = info
                    .string()
                    .map_err(|_| PyRuntimeError::new_err("Failed to get string values"))?;
                if let Some(values) = values {
                    values
                        .iter()
                        .map(|v| std::str::from_utf8(v).unwrap_or(""))
                        .collect::<Vec<_>>()
                        .into_py_any(py)
                } else {
                    let empty: Vec<String> = vec![];
                    empty.into_py_any(py)
                }
            }
            _ => unimplemented!(),
        })
    }

    fn has_info(&self, tag: &str) -> PyResult<bool> {
        let tag_bytes = tag.as_bytes();
        self.record
            .header()
            .info_type(tag_bytes)
            .map(|_id| true)
            .map_err(|_| PyKeyError::new_err(format!("INFO field {} not in header", tag)))
    }
}

#[pymodule]
fn bcf_reader(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyReader>()?;
    m.add_class::<PyRecord>()?;
    m.add_class::<PyHeader>()?;
    Ok(())
}
