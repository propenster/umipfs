use std::fs::File;

use mysql::{prelude::Queryable, Pool};
use ipfs_api::IpfsClient;
use ipns::IpnsClient;
use serde_json::json;
use std::env;

/// Config struct
/// 
/// This has been deprecated now
///
#[deprecated]
struct Config {
    /// * ipfs_url
    ipfs_url: String,
    /// * database_connection_string
    database_connection_string: String,
    /// * ipns_key
    ipns_key: String,
}

impl Config{
    pub fn from_file_path<A: AsRef<Path>>(config_path: A) -> Result<Self, std::error::Error>{
        let json_string = std::fs::read_to_string(&config_path)?;
        serde_json
    }
}
/// retrieve an environment variable
/// 
/// # Arguments
///
/// * 'key' - envVar key e.g MYSQL_CONNECTION_STRING, IPFS_URL, IPFS_USERNAME etc
/// 
/// # Returns
/// 
/// String value of Environment Variable
fn expect_env(key: &str) -> String{
    env::var(key).expect(format!("error finding {}. Ensure you set an environment variable for {}", key, key).as_str())
}

async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mysql_conn_string = expect_env("MYSQL_CONNECTION_STRING");
    let pool = mysql::Pool::new(mysql_conn_string)?;
    let ipfs_client = IpfsClient::default();
    
    //this is our hypothetical table to be retrieved from our MYSQL connection via 
    let table_name = "Table1";
    let table_data = get_table_data(&pool, table_name).await?;
    let table_hash = upload_to_ipfs(&ipfs_client, table_data).await?;

    // Publish the table hash to IPNS
    let _ = ipfs_client.name_publish( &table_hash, false, None, None, None).await?;

    println!("IPNS hash for the table: {}", ipns_key);

    Ok(())
}


/// We extract data about our table rows of data to be hashed
/// 
/// # Arguments
/// 
/// * 'pool' - reference to an open MYSQL connection
/// * 'table_name' - name of the table to be retrieved
/// 
/// # Returns
/// 
/// A vector of data in rows
async fn get_table_data(pool: &Pool, table_name: &str) -> Result<Vec<Vec<Option<String>>>, mysql::Error> {
    let mut conn = pool.get_conn().await?;
    let query = format!("SELECT * FROM {}", table_name);
    let rows: Vec<Vec<Option<String>>> = conn.query_map(query, |row| {
        (0..row.len())
            .map(|i| row.get_opt::<String, _>(i))
            .collect::<Result<Vec<Option<String>>, _>>()
    }).await?;

    Ok(rows)
}
/// this function pushes our hashes to IPFS
/// 
/// # Arguments
/// * 'ipfs' - a reference to an IpfsClient
/// * 'data' - hashed data to be pushed
/// 
/// # Returns
/// 
/// the hash of the add response
async fn upload_to_ipfs(ipfs: &IpfsClient, data: Vec<Vec<Option<String>>>) -> Result<String, ipfs_api::IpfsApiError> {
    let mut ipfs_hashes = Vec::new();

    for row in data {
        let mut row_hash = Vec::new();

        for cell in row {
            let cell_content = cell.unwrap_or_default();
            let result = ipfs.add(cell_content.as_bytes()).await?;
            let hash = result.hash;

            row_hash.push(hash);
        }

        let row_hash_string = ipfs.add(json!(row_hash).to_string().as_bytes()).await?.hash;
        ipfs_hashes.push(row_hash_string);
    }

    let table_hash_string = ipfs.add(json!(ipfs_hashes).to_string().as_bytes()).await?.hash;
    Ok(table_hash_string)
}