# GTFS General
A general to command your GTFS data.
<!-- TOC start (generated with https://github.com/derlin/bitdowntoc) -->

- [Use with Docker (recommended)](#use-with-docker-recommended)
   * [Requirements](#requirements)
- [Native Installation](#native-installation)
   * [Requirements](#requirements-1)
   * [Install](#install)
- [CLI](#cli)
         - [Main](#main)
         - [`gtfs-general docs`](#gtfs-general-docs)
         - [`gtfs-general docs generate`](#gtfs-general-docs-generate)
         - [`gtfs-general extract-bbox`](#gtfs-general-extract-bbox)
         - [`gtfs-general extract-date`](#gtfs-general-extract-date)
         - [`gtfs-general metadata`](#gtfs-general-metadata)
- [Examples](#examples)
- [Credit](#credit)

<!-- TOC end -->


<!-- TOC --><a name="use-with-docker-recommended"></a>
## Use with Docker (recommended)
<!-- TOC --><a name="requirements"></a>
### Requirements
- Docker
- Linux (If you want to use the `docker_run.sh` convenience script)

Notes:
- The `docker_run.sh` script is a convenience script to run the docker container with the correct parameters.
- The `docker_run.sh` script is only tested on Linux.
- All files should be placed in the folder where the `docker_run.sh` script is located and executed.

Setup the project and download the test file:
```bash
# Get the project
git clone git@github.com:MichaelsJP/gtfs-general.git
cd gtfs-general
# Make the script executable
chmod +x docker_run.sh
# Download the test gtfs file
wget https://download.gtfs.de/germany/rv_free/latest.zip -O gtfs-germany-rv-latest.zip
```

Example: Get the service range of the test gtfs file:
```bash
# Get the metadata
./docker_run.sh metadata --input-object gtfs-germany-rv-latest.zip
```

Example extract a bounding box from the gtfs file:
```bash
# Get a fitting bbox with format `CSV` from https://boundingbox.klokantech.com
# Query the GTFS file with a specific bbox. The bbox is in format "lon min, lat min, lon max, lat max"
./docker_run.sh extract-bbox --input-object gtfs-germany-rv-latest.zip --output-folder output/bbox-gtfs-germany-rv-latest --bbox "7.5117,47.5325,10.4956,49.7913"
```

Example extract a date range from the gtfs file:
```bash
# Query the date range of the GTFS file. The date will be in format "YYYY-MM-DD HH:MM:SS"
./docker_run.sh metadata --input-object gtfs-germany-rv-latest.zip
# Query the GTFS file with a specific date range fitting to the actual range from metadata in the format "YYYYMMDD"
./docker_run.sh extract-date --input-object gtfs-germany-rv-latest.zip --output-folder output/range-gtfs-germany-rv-latest --start-date "20240401" --end-date "20240501"
```


<!-- TOC --><a name="native-installation"></a>
## Native Installation


<!-- TOC --><a name="requirements-1"></a>
### Requirements
- Python >= 3.9 && < 2.13
- Poetry

<!-- TOC --><a name="install"></a>
### Install
```bash
# Create venv
python -m venv venv
# Activate venv
source venv/bin/activate
# Install the packages
poetry install
# Run with poetry
poetry run gtfs-general --help
# Run through python and mind the _
python -m gtfs_general --help
```

<!-- TOC --><a name="cli"></a>
## CLI

<!-- TOC --><a name="main"></a>
##### Main

**Usage**:

```console
$ gtfs-general [OPTIONS] COMMAND [ARGS]...
```

**Options**:

* `--logging TEXT`: [default: INFO]
* `--cores INTEGER`: Set the number of cores to use for processing.  [default: 14]
* `--progress / --no-progress`: Deactivate the progress bars.  [default: progress]
* `-v, --version`: Show the application's version and exit.
* `--install-completion`: Install completion for the current shell.
* `--show-completion`: Show completion for the current shell, to copy it or customize the installation.
* `--help`: Show this message and exit.

**Commands**:

* `docs`: Generate documentation
* `extract-bbox`
* `extract-date`
* `metadata`

<!-- TOC --><a name="gtfs-general-docs"></a>
##### `gtfs-general docs`

Generate documentation

**Usage**:

```console
$ gtfs-general docs [OPTIONS] COMMAND [ARGS]...
```

**Options**:

* `--help`: Show this message and exit.

**Commands**:

* `generate`: Generate markdown version of usage...

<!-- TOC --><a name="gtfs-general-docs-generate"></a>
##### `gtfs-general docs generate`

Generate markdown version of usage documentation

**Usage**:

```console
$ gtfs-general docs generate [OPTIONS]
```

**Options**:

* `--name TEXT`: The name of the CLI program to use in docs.
* `--output FILE`: An output file to write docs to, like README.md.
* `--help`: Show this message and exit.

<!-- TOC --><a name="gtfs-general-extract-bbox"></a>
##### `gtfs-general extract-bbox`

**Usage**:

```console
$ gtfs-general extract-bbox [OPTIONS]
```

**Options**:

* `--input-object TEXT`: Directory or zip File from which the GFTS files are read  [required]
* `--output-folder TEXT`: Directory to which the GFTS files are written  [required]
* `--bbox TEXT`: The bbox for selecting the GTFS data to keep. Format is WGS84 Coordinates lon/lat (lon min, lat min, lon max, lat max) Example: "8.573179,49.352003,8.79405,49.459693"  [required]
* `--help`: Show this message and exit.

<!-- TOC --><a name="gtfs-general-extract-date"></a>
##### `gtfs-general extract-date`

**Usage**:

```console
$ gtfs-general extract-date [OPTIONS]
```

**Options**:

* `--input-object TEXT`: Directory or zip File from which the GFTS files are read  [required]
* `--output-folder TEXT`: Directory to which the GFTS files are written  [required]
* `--start-date TEXT`: Lower date boundary. Format: YYYYMMDD. e.g. 20221002 for 2nd October 2022  [required]
* `--end-date TEXT`: Lower date boundary. Format: YYYYMMDD. e.g. 20221002 for 2nd October 2022  [required]
* `--help`: Show this message and exit.

<!-- TOC --><a name="gtfs-general-metadata"></a>
##### `gtfs-general metadata`

**Usage**:

```console
$ gtfs-general metadata [OPTIONS]
```

**Options**:

* `--input-object TEXT`: Directory or zip File from which the GFTS files are read  [required]
* `--help`: Show this message and exit.

<!-- TOC --><a name="examples"></a>
## Examples

Ask for help
```bash
poetry run gtfs-general --help
```

Show metadata (for now just service days)
```bash
poetry run gtfs-general metadata --input-object [zip/folder]
```

Cut by bounding box
```bash
# Bounding box with WGS84 4326 Coordinates lon/lat (lon min, lat min, lon max, lat max):
poetry run gtfs-general extract-bbox --input-object [zip/folder] --output-folder output --bbox "8.573179,49.352031,8.794049,49.459693"
```

Cut by date
```bash
# Dates in format "YYYYMMDD"
python -m gtfs_general extract-date --input-object [zip/folder] --output-folder  --start-date "20220601" --end-date "20220701"
```

<!-- TOC --><a name="credit"></a>
## Credit
This tool was inspired by https://github.com/gberaudo/gtfs_extractor
