import pytest
from bcf_reader import PyReader

def test_bcf_reader_basics():
    # Open the test VCF file
    reader = PyReader("tests/test.vcf.gz")
    
    # Test iteration and basic properties
    for record in reader:
        # Test chromosome access
        assert isinstance(record.chrom, str)
        
        # Test position properties
        assert record.pos >= 0
        assert record.start >= 0
        assert record.end >= record.start
        
        # Break after first record to keep test fast
        break

def test_bcf_info_fields():
    reader = PyReader("tests/test.vcf.gz")
    
    for record in reader:
        # Test info field presence check
        if record.has_info("AF"):
            af = record.info("AF")
            print(af)
            assert isinstance(af, list)
            assert all(isinstance(x, float) for x in af)
        
        # Test non-existent field
        with pytest.raises(KeyError):
            record.has_info("NONEXISTENT_FIELD")
        
        break

def test_bcf_record_iteration():
    reader = PyReader("tests/test.vcf.gz")
    records = []
    
    # Collect first few records
    for i, record in enumerate(reader):
        records.append((record.chrom, record.pos))
        if i >= 2:  # Just test first 3 records
            break
    
    assert len(records) > 0
    # Verify positions are ordered
    for i in range(1, len(records)):
        if records[i][0] == records[i-1][0]:  # Same chromosome
            assert records[i][1] >= records[i-1][1]  # Position should be >= previous 

def test_header_functionality_and_set_info():
    reader = PyReader("tests/test.vcf.gz")
    header = reader.header()
        
        # Test adding new INFO field
    header.add_info({
        "ID": "TEST_INFO",
        "Number": "1",
        "Type": "Float",
        "Description": "Test info field"
    })
    
    # Test adding another INFO field
    header.add_info({
        "ID": "TEST_FLAG",
        "Number": "0",
        "Type": "Flag",
        "Description": "Test flag field"
    })
    assert "ID=TEST_INFO" in str(reader.header())
    assert "ID=TEST_FLAG" in str(reader.header())

    # Test samples access
    samples = header.samples()
    assert isinstance(samples, list)
    assert all(isinstance(s, str) for s in samples)
    
    # Get first record and its header
    for record in reader:
        
        record.set_info("TEST_INFO", [1.0])
        record.set_info("TEST_FLAG", [True])

        print(record)


        
        break

if __name__ == "__main__":
    pytest.main(["-s", "--capture=no", __file__])
