# GTFS General
A general to command your GTFS data.

<!-- TOC depthFrom:1 depthTo:3 withLinks:1 updateOnSave:0 orderedList:0 -->
- [Requirements](#requirements)
- [Install](#install)
- [CLI](#cli)
      - [Main](#main)
      - [`gtfs-general docs`](#-gtfs-general-docs-)
      - [`gtfs-general docs generate`](#-gtfs-general-docs-generate-)
      - [`gtfs-general extract-bbox`](#-gtfs-general-extract-bbox-)
      - [`gtfs-general extract-date`](#-gtfs-general-extract-date-)
      - [`gtfs-general metadata`](#-gtfs-general-metadata-)
- [Examples](#examples)
- [Credit](#credit)
<!-- /TOC -->

## Requirements
- Python >= 3.8
- Poetry

## Install
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

## CLI

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

##### `gtfs-general metadata`

**Usage**:

```console
$ gtfs-general metadata [OPTIONS]
```

**Options**:

* `--input-object TEXT`: Directory or zip File from which the GFTS files are read  [required]
* `--help`: Show this message and exit.

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

## Credit
This tool was inspired by https://github.com/gberaudo/gtfs_extractor
