import pandas as pd
import os
import sys
# python3 tbl_to_csv.py ./data/tpch/data  ./


def get_headers(file_name):
    headers = []
    if file_name.startswith("nation"):
        headers = ['n_nationkey', 'n_name', 'n_regionkey', 'n_comment'] 
    elif file_name.startswith("region"):
        headers = ['r_regionkey', 'r_name', 'r_comment']  
    elif file_name.startswith("orders"):
        headers = ['o_orderkey', 'o_custkey', 'o_orderstatus', 'o_totalprice', 'o_orderdata', 'o_orderpriority', 'o_clerk', 'o_shippriority', 'o_comment']  
    elif file_name.startswith("lineitem"):
        headers = ['l_orderkey', 'l_partkey', 'l_suppkey', 'l_linenumber', 'l_quantity', 'l_extendedprice', 'l_discount', 'l_tax', 'l_returnflag', 'l_linestatus', 'l_shipdate', 'l_commitdata', 'l_receiptdata', 'l_shipinstruct', 'l_shipmode', 'l_comment']  # Replace with your actual headers
    elif file_name.startswith("partsupp"):
        headers = ['ps_partkey', 'ps_suppkey', 'ps_availqty', 'ps_supplycost', 'ps_comment']  
    elif file_name.startswith("customer"):
        headers = ['c_custkey', 'c_name', 'c_address', 'c_nationkey', 'c_phone', 'c_acctbal', 'c_mktsegment', 'c_comment']  
    elif file_name.startswith("supplier"):
        headers = ['s_suppkey', 's_name', 's_address', 's_nationkey', 's_phone', 's_acctbal','s_comment']  
    elif file_name.startswith("part"):
        headers = ['p_partkey', 'p_name', 'p_mfgr', 'p_brand', 'p_type', 'p_size', 'p_container', 'p_retailprice', 'p_comment']  
      
    return headers

def process_data(dataset, folder_path, output_dir):
    if "tpch" in dataset:
        # Iterate over files in the folder
        for file_name in os.listdir(folder_path):
            if file_name.endswith('.tbl'):
                file_path = os.path.join(folder_path, file_name)

                # Read the .tbl file into a pandas DataFrame
                df = pd.read_table(file_path, delimiter='|', header=None)

                # Drop the last column
                df = df.iloc[:, :-1]

                print(f"file and number of rows:{file_name},{len(df)}")

                # Generate the output file name
                output_file = os.path.splitext(file_name)[0] + '.csv'

                # Specify the headers
                headers = get_headers(file_name)

                # Write the DataFrame to a .csv file with headers
                if not os.path.exists(output_dir):
                    os.makedirs(output_dir)
                df.to_csv(os.path.join(output_dir, output_file), index=False, header=headers)
    elif "soccer" in dataset:
        for file_name in os.listdir(folder_path):
            print(file_name)
            file_path = os.path.join(folder_path, file_name)
            if file_name == "Team.csv":
                df = pd.read_csv(file_path)
                df['team_fifa_api_id'] = pd.to_numeric(df['team_fifa_api_id'], errors='coerce').astype('Int64')
                df.to_csv(os.path.join(output_dir, "HomeTeam.csv"), index=False, header=True)
                df.to_csv(os.path.join(output_dir, "AwayTeam.csv"), index=False, header=True)
                os.remove(file_path)
            elif file_name == "Match.csv":
                df = pd.read_csv(file_path)
                df['match_api_id'] = pd.to_numeric(df['match_api_id'], errors='coerce').astype('Int64')
                df['home_team_api_id'] = pd.to_numeric(df['home_team_api_id'], errors='coerce').astype('Int64')
                df['away_team_api_id'] = pd.to_numeric(df['away_team_api_id'], errors='coerce').astype('Int64')
                df.to_csv(os.path.join(output_dir, "Match.csv"), index=False, header=True)
            elif file_name == "Country.csv":
                df = pd.read_csv(file_path)
                df['id'] = pd.to_numeric(df['id'], errors='coerce').astype('Int64')
                df.to_csv(os.path.join(output_dir, "Country.csv"), index=False, header=True)
            elif file_name == "League.csv":
                df = pd.read_csv(file_path)
                df['id'] = pd.to_numeric(df['id'], errors='coerce').astype('Int64')
                df['country_id'] = pd.to_numeric(df['country_id'], errors='coerce').astype('Int64')
                df.to_csv(os.path.join(output_dir, "League.csv"), index=False, header=True)
            elif file_name not in ["Match.csv", "League.csv", "Country.csv", "HomeTeam.csv", "AwayTeam.csv"]:
                os.remove(file_path)
    else:
        ValueError("Unknown dataset")

if __name__ == "__main__":
    dataset = sys.argv[1]
    input_dir = sys.argv[2]
    output_dir = sys.argv[3]

    process_data(dataset, input_dir, output_dir)


