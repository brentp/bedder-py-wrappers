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
    print(header.samples())
        
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
    assert "ID=TEST_INFO" in str(header)
    assert "ID=TEST_FLAG" in str(header)

    # Test samples access
    samples = header.samples()
    assert isinstance(samples, list)
    assert all(isinstance(s, str) for s in samples)

    import time
    
    # Get first record and its header
    for record in reader:
        record.translate(header) # normally bedder would call this before user accesses the record
        record.set_info("TEST_INFO", [1.0])
        record.set_info("TEST_FLAG", True)

        n = 100_000
        t = time.time()
        for i in range(n):
            af = record.info("AF")
            assert af is not None
            DP = record.info("DP")
            assert DP[0] > 0
        print(f'\n!!!>   accessed {2*n} info records in {time.time() - t:.2f} seconds. records per second: {2 * n / (time.time() - t):.0f}\n')
        print(record)


        

if __name__ == "__main__":
    pytest.main(["-s", "--capture=no", __file__])
