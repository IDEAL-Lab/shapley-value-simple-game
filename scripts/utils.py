import configparser
import os
import fnmatch
import re
import polars as pl

def read_config(filename):
    config = configparser.ConfigParser()
    
    # The read() function returns a list of successfully read files
    files = config.read(filename)

    # Check if the file was read successfully
    if not files:
        raise FileNotFoundError(f"Failed to read file at {filename}")
    
    # If we get to this point, the file was read successfully
    # so we can return the config object
    return config

def get_config(dataset):
    result = {}

    config = read_config("./scripts/config.cfg")

    ### General
    config_dict = dict(config.items("general"))
    default_alpha = config_dict.get("default_alpha")
    default_alpha = float(default_alpha)
    result["default_alpha"] = default_alpha

    default_beta = config_dict.get("default_beta")
    default_beta = float(default_beta)
    result["default_beta"] = default_beta

    default_number_of_data_owner = config_dict.get("default_number_of_data_owner")
    default_number_of_data_owner = int(default_number_of_data_owner)
    result["default_number_of_data_owner"] =  default_number_of_data_owner

    default_max_copy = config_dict.get("default_max_copy")
    default_max_copy = int(default_max_copy)
    result["default_max_copy"] = default_max_copy
    
    default_owner_for_small_table = config_dict.get("default_owner_for_small_table")
    default_owner_for_small_table = int(default_owner_for_small_table)
    result["default_owner_for_small_table"] = default_owner_for_small_table

    default_basic_owner = config_dict.get("default_basic_owner")
    default_basic_owner = int(default_basic_owner)
    result["default_basic_owner"] = default_basic_owner

    timeout = config_dict.get("timeout")
    timeout = int(timeout)
    result["timeout"] = timeout

    default_sample_size = config_dict.get("default_sample_size")
    default_sample_size = int(default_sample_size)
    result["default_sample_size"] = default_sample_size
    config_dict = dict(config.items(dataset))

    for table in get_tables(dataset):
        number_of_record = config_dict.get(table)
        number_of_record = int(number_of_record)
        result[table] = number_of_record

    return result

def get_tables(dataset):
    if dataset == "tpch":
        return ["customer","lineitem", "nation", "orders", "part", "partsupp", "region", "supplier"]
    else:
        ValueError("Unkown data set!")

def create_new_iters(output_dir):
    # check if the output dir already exists or not
    if os.path.isdir(output_dir):
        # yes, the folder exists
        files = os.listdir(output_dir)
        number_of_json_files = len(fnmatch.filter(os.listdir(output_dir), '*.json'))
        if number_of_json_files!= 16:
            print(f"!!!!!!!!!!!!!remove files in output_dir!!!!!!!!!! {output_dir}")
            for file in files:
                os.remove(f"{output_dir}/{file}")
        else:
            print(f"~~~~~~~~~~~skip output_dir~~~~~~~~~~~~~~: {output_dir}")
            return False
    else:
        print(f"======create output_dir========: {output_dir}")
        os.makedirs(output_dir)
    
    return True

def print_owners_and_records(data, table):
    owners_and_reocords = pl.DataFrame(data)
    q = (
        owners_and_reocords.lazy()
        .groupby("owner")
        .agg(
            pl.count("owner")  # Renamed column using the `as` parameter
        )
    )

    df = q.collect()
    # print(owners_and_reocords.columns)
    print(f"table and number of data owners: {table}, {len(df['owner_count'].to_list())} ")
