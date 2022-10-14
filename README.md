# GTFS General
A general to command your GTFS data.

### Supported operations
Extract subdata from your GTFS feed:
- Show metadata
- Extract by bounding box
- Extract by date


### Requirements
- Python >= 3.8
- Poetry

### Install
```bash
# Create a virtual environment
poetry install
python -m gtfs-general --help
```

### Examples
Ask for help
```bash
python -m gtfs-general --help
```

Show metadata (for now just service days)
```bash
python -m gtfs_general metadata --input-object [zip/folder]
```

Cut by bounding box
```bash
# Bounding box with WGS84 4326 Coordinates lon/lat (lon min, lat min, lon max, lat max):
python -m gtfs-general extract-bbox --input-object [zip/folder] --output-folder output --bbox "8.573179,49.352031,8.794049,49.459693"
```

Cut by date
```bash
# Dates in format "YYYYMMDD"
python -m gtfs_general extract-date --input-object [zip/folder] --output-folder  --start-date "20220601" --end-date "20220701"
```

### Credit
This tool was inspired by https://github.com/gberaudo/gtfs_extractor
