# Fast Shapley Value Computation in Data Assemblage Tasks as Cooperative Simple Games

## Install dependencies
* OS: Ubuntu 20.04 LTS.
* Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`

## Build

```bash
cargo build --release
```

## Generate source data
We use two data sets in our experiment.
- TPC-H: a benchmark data set that lacks data owner information
	```bash
	git submodule update --init --recursive
	./scripts/compile-tpch.sh
	./scripts/tpch-dbgen.sh -s 1.0
	```
	After this step, we can find the source data in the folder 	"./data/tpch/data".
- ESD (European Soccer Database): a real-world data set that contains data owner information. 
	- We can manuallly export a csv verison of the data set from the [sqlite database](https://www.kaggle.com/datasets/hugomathien/soccer) or download a csv version directly [here](https://www.kaggle.com/datasets/abdelrhmanragab/european-soccer-database).
	- Save the csv files to the folder "./data/soccer/data"
	- Run command: ./scripts/soccer-dbgen.sh 
## Generate assignment data
We skip this step for the ESD data set since it contains data owner information. 
For TPC-H data set, assign source data to data owners and store the assignment via:
```bash
python3 ./scripts/assign_data.py -d <dataset> -a <alpha> -b <beta> -k <number_of_data_owner> -m <max_copy> -o <equal owners> -r <equal records> -f <output dir>
```
Example:
```bash
python3 ./scripts/assign_data.py -d tpch -a 3.0 -b 3.0 -k 500 -m 4 -o 1 -r 1 -f ./data/tpch/assignment
```
After this step, we can find the assignment data in the folder "./data/tpch/assignment".

## Compute Shapley value
```bash
 cal_sv  -d <dataset>  -c <source data dir> -a <data assignment dir> -o <output file> -m <method>
```
Example:
- TPC-H: 
```bash
./target/release/cal_sv -d tpch -c data/tpch/data -a data/tpch/assignment -o rdsv.json -m rdsv
```
- ESD:
```bash
./target/release/cal_sv -d soccer -c data/soccer/data -o rdsv.json -m rdsv
```

## Compute Shapley value with ablation
Calculate Shapley value for all data owners by ablating one type of decomposition via:
```bash
 cal_sv_ablation  -d <dataset>  -c <source data dir> -a <data assignment dir> -o <output file> -m <method> --ablation <ablation_type>
```
Example:
- TPC-H: 
```bash
./target/release/cal_sv_ablation -d tpch -c data/tpch/data -a data/tpch/assignment -o rdsv.json -m rdsv --ablation no-horizontal
```
- ESD:
```bash
./target/release/cal_sv_ablation -d soccer -c data/soccer/data -o rdsv.json -m rdsv --ablation no-horizontal
```