# dinf - Director Info CLI

A very simple, minimalist directory information CLI.

The main goal is to get the number of files, total size, and a high-level overview (biggest files and size per extension).

## Usage

```sh
# simple info (will scan from current dir and list the top 5 biggest files)
dinf

# set number of biggest files
dinf -n 12

# display summary only (number of files and total size)
dinf -s

# will run dinf on two folders and display results back to back
dinf some/path another/path

# just get the summary of those two directories
dinf -s some/path another/path
```