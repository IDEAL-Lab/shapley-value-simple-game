import sys, getopt
import numpy as np
from random import sample, shuffle
import pandas as pd
import os
import json

from utils import get_config, get_tables, print_owners_and_records

def print_usage():
    print('assign_data.py -d dataset -a <alpha> -b <beta> -k <number_of_data_owner> -m <max_copy> -o <equal owners> -r <equal records> -f <output_dir>')

def get_parameter(argv):
    try:
        # opts is a list of returning key-value pairs, args is the options left after striped
        # the short options 'hi:o:', if an option requires an input, it should be followed by a ":"
        # the long options 'ifile=' is an option that requires an input, followed by a "="
        opts, args = getopt.getopt(argv, "hd:a:b:k:m:o:r:f:", ["dataset", "alpha=", "beta=", "number_of_data_owner=", "max_copy=", "assign_owner_equally", "assign_record_equally", "output_dir"])
    except getopt.GetoptError:
        print("error in get opts, args")
        print_usage()
        sys.exit(2)

    if not opts:
        print("error in getting opts")
        sys.exit(2)

    # print arguments
    for opt, arg in opts:
        if opt == '-h':
            print_usage()
            sys.exit(2)
        elif opt in ("-d", "--dataset"):
            dataset = arg
        elif opt in ("-a", "--alpha"):
            alpha = float(arg)
        elif opt in ("-b", "--beta"):
            beta = float(arg)
        elif opt in ("-k", "--number"):
            number_of_data_owner = int(arg)
        elif opt in ("-m", "--copy"):
            max_copy = int(arg)
        elif opt in ("-o", "--equalowners"):
            assign_owner_equally = bool(int(arg))
        elif opt in ("-r", "--equalrecords"):
            assign_record_equally = bool(int(arg))
        elif opt in ("-f", "--output"):
            output_dir = arg

    return (dataset, alpha, beta, number_of_data_owner, max_copy, assign_owner_equally, assign_record_equally, output_dir)

def assign_data(dataset, alpha, beta, number_of_data_owner, max_copy, assign_owner_equally, assign_record_equally, output_dir):
    config = get_config(dataset)
    default_owner_for_small_table = config.get("default_owner_for_small_table")

    print(f"settings: dataset-{dataset}, assign_owner_equally-{assign_owner_equally}, assign_record_equally-{assign_record_equally}")
    print(f"alpha-{alpha}, beta-{beta}, number_of_data_owner-{number_of_data_owner}, default_owner_for_small_table-{default_owner_for_small_table}")
    print(f"output_dir: {output_dir}")

    if assign_owner_equally: 
        assign_owner_mode = "equalowner"
    else:
        assign_owner_mode = "inequalowner"
    
    if assign_record_equally:
        assign_record_mode = "equalrecord"
    else:
        assign_record_mode = "inequalrecord"

    if not os.path.exists(output_dir):
         os.makedirs(output_dir)
         
    tables = get_tables(dataset)

    owners = assign_owners_to_tables(beta, assign_owner_equally, number_of_data_owner, dataset, tables, default_owner_for_small_table)

    for index in range(len(tables)): 
        # generate owner list
        assign_records_to_owners(alpha, owners[index], max_copy, assign_record_equally, dataset, tables[index], output_dir) 

def assign_owners_to_tables(beta, assign_owner_equally, number_of_data_owner, dataset, table, default_owner_for_small_table):
    if assign_owner_equally:
        return assign_owners_to_tables_equally(dataset, number_of_data_owner, table, default_owner_for_small_table)
    else:
        return assign_owners_to_tables_inequally(beta, number_of_data_owner, dataset, table)

def assign_owners_to_tables_equally(dataset, number_of_data_owner, tables, default_owner_for_small_table):
    table_owners_count_list = []
    
    if dataset.startswith("tpch"):
        owners_count_list = []
        for table in tables:
            if table == "region" or table == "nation":
                owners_count_list.append(default_owner_for_small_table)
            else:
                owners_count_list.append(number_of_data_owner)
        table_owners_count_list = np.array(owners_count_list)
    
    table_owners_count_list = table_owners_count_list.astype(int)
    sorted_table_to_owners_list = []
    total_owners = range(number_of_data_owner)
    for i in range(len(table_owners_count_list)):
        owners = sample(total_owners, table_owners_count_list[i])
        sorted_table_to_owners_list.append(owners)
    
    return sorted_table_to_owners_list

def assign_owners_to_tables_inequally(beta, number_of_data_owner, dataset, tables):
    config = get_config(dataset)

    default_basic_owner = config.get("default_basic_owner")

    table_owners_count_list = []
     
    if dataset.startswith("tpch"):
        owners_count_list = []
        for table in tables:
            if table == "region" or table == "nation":
                owners_count_list.append(default_owner_for_small_table)
            elif table == 'lineitem':
                owners_count_list.append(number_of_data_owner)
            else:
                owners_count_list.append(default_basic_owner)
        table_owners_count_list = np.array(owners_count_list)  
    
    
    table_owners_count_list = table_owners_count_list.astype(int)
    sorted_table_to_owners_list = []
    total_owners = range(number_of_data_owner)
    for i in range(len(table_owners_count_list)):
        owners = sample(total_owners, table_owners_count_list[i])
        sorted_table_to_owners_list.append(owners)
    
    return sorted_table_to_owners_list

def assign_records_to_owners(alpha, owners, max_copy, assign_record_equally, dataset, table, output_dir):
    (data, records) = assign_records_to_owners_for_table(alpha, owners, max_copy, dataset, table, assign_record_equally)
    
    save_metadata_to_dir(output_dir, data, records, table)
    print_owners_and_records(data, table)


def assign_records_to_owners_for_table(alpha, owners, max_copy, dataset, table, assign_record_equally):
    config = get_config(dataset)
    number_of_records = int(config.get(table))

    number_of_records_per_copy = get_number_of_records_by_zipfian(alpha, 
                                                    min(len(owners),max_copy), 
                                                    number_of_records)    
    record_list = []
    data_owner_list = []
    
    pvals = get_zipfian(beta, len(owners))
    # Assign each record to owners
    record_index = 0
    current_number_of_copy = 1
    for count in number_of_records_per_copy:
        for i in range(0, count):
            if assign_record_equally: 
                owners_for_a_record = sample(owners, min(len(owners), current_number_of_copy))
            else:
                owners_for_a_record = np.random.choice(owners, current_number_of_copy, False, pvals)

            for owner in owners_for_a_record:
                record_list.append(record_index)
                data_owner_list.append(owner)
            record_index += 1
        current_number_of_copy += 1
    
    records = [i for i in range(number_of_records)]
    shuffle(records)
    
    assign_records_to_owners_without_records(owners, data_owner_list)

    data = {"index": record_list, "owner": data_owner_list}
    
    return (data, records)

def assign_records_to_owners_without_records(owner_list, data_owner_list):
    owners_without_records = list(set(owner_list) - set(data_owner_list))
    len_before = len(data_owner_list)

    num_of_records = len(data_owner_list)

    while len(owners_without_records) > 0:
        record_to_owner_map = {}

        # map each owner to the record index
        for index in range(num_of_records):
            current_owner = data_owner_list[index]
            if current_owner not in record_to_owner_map:
                record_to_owner_map[current_owner] = []
            record_to_owner_map.get(current_owner).append(index)

        filtered_record_to_owner_map = {k:v for (k,v) in record_to_owner_map.items() if len(v) >= 2}
        owners_with_more_than_one_record = list(filtered_record_to_owner_map.keys())
        
        owners_assigned = []
        current_index = 0
        
        for index in range(len(data_owner_list)):
            if data_owner_list[index] == owners_with_more_than_one_record[current_index]:
                owner_to_assign = owners_without_records[current_index]
                data_owner_list[index] = owner_to_assign
                owners_assigned.append(owner_to_assign)

                current_index += 1
                if current_index == len(owners_without_records) or current_index == len(owners_with_more_than_one_record):
                    break
                    
        if len(owners_without_records) >= len(owners_assigned):
            owners_without_records = list(set(owners_without_records) - set(owners_assigned))

    len_after = len(data_owner_list)
    assert(len_before == len_after)

def save_metadata_to_dir(output_dir, data, records, table):
    record_to_owner_df = pd.DataFrame(data)
    record_to_owner_df.to_json(f'{output_dir}/{table}-owner.json')
    
    with open(f'{output_dir}/{table}-index.json', 'w') as json_file:
        json.dump(records, json_file)

    return 

def get_number_of_records_by_zipfian(alpha_zipfian, number_of_copy, number_of_records):
    pvals = get_zipfian(alpha_zipfian, number_of_copy)

    number_of_records_per_copy = np.array(pvals) * number_of_records
    number_of_records_per_copy = number_of_records_per_copy.astype(int)
    total_count = np.sum(number_of_records_per_copy)

    if total_count < number_of_records:
        number_of_records_per_copy[0] = number_of_records_per_copy[0] +  number_of_records - total_count
    
    return number_of_records_per_copy

def get_zipfian(a, k):
    probs = np.zeros(k)
    for i in range(1, k+1):
        probs[i-1] = 1/pow(i,a)
    
    total_prob = sum(probs)
    probs = probs/total_prob
    return probs


if __name__ == "__main__":
    # print(sys.argv[1:])

    if len(sys.argv) != 17:
        print("incorrect number of inputs")
        print_usage()
        sys.exit(1)
    
    (dataset, alpha, beta, number_of_data_owner, max_copy, assign_owner_equally, assign_record_equally, output_dir) = get_parameter(sys.argv[1:])
    assign_data(dataset, alpha, beta, number_of_data_owner, max_copy, assign_owner_equally, assign_record_equally, output_dir)