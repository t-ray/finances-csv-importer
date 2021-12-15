## So what is this exactly?
This is a personal project to import records of my financial transactions into a database. 
This repo, specifically, reads csv files that adhere to a specific format into a postgres
database of a specific schema. That's the basic functionality. It can either import
individual files, or it can read a directory and import all files within the directory.

Once the rows are in the database, they can be queried however you want; you're not limited
to canned reports included in Tool X. 
## But, WHY?
I recently experienced a disruption in my life where I needed to learn new skills; namely 
the ability to manage my own finances. 

Yes, there are plenty of apps and websites that already do this. Each is much more mature and has
both greater depth and breadth. But I wanted to learn these skills
myself, while also at the same time scratching the itch of having a side project that actually interested
me.

## Why rust?
Because rust is **awesome**, and I don't have the freedom of using it at my day job any longer. Plus, this is 
primarily a command-line utility, so rust seemed an obvious choice.

## Database
Right now, only postgres is supported. Version 11 has been used for development, but it's possible older
versions would be supported. 

The importer loads all rows into a single table (named `transactions` by default). It is intended that
this table have transactions from different accounts. The table shaped has intentionally been generalized
to express transactions from a traditional checking account, as well as credit cards. 

### Database Schema
The database schema follows the same general shape of a few spreadsheets that I have been 
maintaining as my own personal ledger. This schema is rigid and highly opinionated. 

The schema is defined as follows [source](resources/schemas/v1__base.sql):
```sql
CREATE TABLE transactions (
  account TEXT NOT NULL,
  tx_id INTEGER NOT NULL,
  tx_date DATE NOT NULL,
  amount NUMERIC(13,4) NOT NULL,
  balance NUMERIC(13,4) NOT NULL,
  vendor TEXT NOT NULL,
  digits TEXT NULL,
  transaction_type TEXT NOT NULL,
  category TEXT NULL,
  subcategory TEXT NULL,
  notes TEXT NULL,
  PRIMARY KEY (account, tx_id)
);
```

Or:

| Column | Type | Description |
| ------ | ---- | ----------- |
| account | TEXT | The abbreviated label of the account; example, BOFA |
| tx_id | INTEGER | A monotonically increasing transaction identifier, within the scope of a single account |
| tx_date | DATE | The date portion of when the transaction posted. |
| amount | NUMERIC | The amount of the transaction. |
| balance | NUMERIC | The balance of the account after the transaction. |
| vendor | TEXT | The vendor with whom the transaction was conducted. |
| digits | TEXT | The last X number of digits of the card used, if any. Useful for tracking transactions within the same account after a card has been re-issued. |
| transaction_type | TEXT | The type of the transaction. This is a simple TEXT column that is not limited to any particular enumerated value. It's really dictated by whatever is input into the source data. Examples could be `Charge`, `Payment`, `Debit`, `Deposit`, `Transfer`, `ACH`, etc. |
| category | TEXT | A human readable category for the transaction. Again, no referential integrity here. Driven by the input data. But it will allow transactions to be grouped together. |
| subcategory | TEXT | An optional subcategory, providing further aggregation possibilities. |
| notes | TEXT | Any free form notes about the transaction. Example: `Birthday gift for Joe.` |

The primary key is a composite of (account, tx_id). 

### Schema / Database Generation
The current version of this importer requires that the database already exists, and the schema has already been applied.
It accepts database connection properties as command line arguments, including table name. So the tool supports 
importing to a different table, as long as it adheres to the same schema.

### NUMERIC(13,4) columns
Other precisions could be chosen; this precision seemed like a sensible default
for typical banking transactions for personal accounts in the US. Feel free to play with other 
precisions.

## CSV Files
The importer can import rows from either a single file (using `--file`), or all csv files 
in a directory (using `--directory`). If importing a directory, the importer specifically filters for 
"csv" files, case-sensitive. 

All files are expected to have the same schema, and the columns closely resemble the database schema. 
I personally keep my ledger in a Google Sheet, and then export to CSV, so the format is informed by that tool. 
I assume a different spreadsheets (i.e., Excel) might output differently formatted values. An example is available
[here](resources/csv/example.csv).

**Warning** The importer assumes the first row is a header row. If your spreadsheet doesn't include a header row, the first row
will not be imported!

The following is the example spreadsheet:
```csv
ACCOUNT,ID,Date, Amount , Balance ,Vendor,Digits,Type,Category,Subcategory,Notes
BOFA8556,00000001,9/22/2021, $ 100.00 , $ 100.00 ,Self,3333,Deposit,Income,,Initial Deposit
BOFA8556,00000002,9/23/2021, $ 50.00 , $ 50.00 ,Bank Of Ameria,3333,Transfer,Savings,,
BOFA8556,00000003,9/23/2021, $ 12.33 , $ 37.67 ,Kroger,3333,Debit,Groceries,,
BOFA8556,00000004,9/24/2021, $ 7.49 , $ 30.18 ,HBO Max,3333,Debit,Entertainment,TV,
BOFA8556,00000005,9/24/2021, $ 20.00 , $ 10.18 ,Best Buy,3333,ACH,Credit Cards,,
BOFA8556,00000006,9/24/2021, $ (75.00), $ 85.18 ,Facebook Marketplace,3333,Deposit,Income,,Sold that old TV
```
The importer makes no calculations; it imports the amount and balances verbatim. The source spreadsheet is assumed to 
be using proper formulae. 

### Date Formats
The importer assumes a US data format in the form of MM/DD/YYYY. Dates are parsed in `domain/parse_date_time`.

### Currency Format
The importer utilizes its own currency implementation, naively assuming a whole number and fixed number of digits. 
Since the importer itself is not performing any calculations, these assumptions are sufficient for formatting into
an argument that can be inserted into the database. It does, however, assume values in parentheses are negative values.

## Import Behavior
The importer will try its best to parse each row (excluding the header row) into its internal row representation. 
Anything that cannot be parsed will be logged and skipped; an unparseable row does not terminate execution. 

After rows have been parsed, they are inserted into the target table. Rows are inserted in *chunks* 
of 50 rows at a time (this value is hard-coded). Rows are inserted as individual statements and not executed in batch.
Batch insertion is a prime candidate for improvement. 

Row insertion uses the syntax `INSERT INTO ... ON CONFLICT DO NOTHING`; so re-importing the same file
again and again is not a destructive operation. 

# Code Structure
There is the entrypoint, `main.rs`, and only a handful of modules: 
* config.rs - defines and parses the command line arguments (and supports environment variables)
* currency.rs - internal implementation of the US based currency used in the csv files
* db.rs - postgres-specific (via [sqlx](https://github.com/launchbadge/sqlx)) code to insert rows into the database
* domain.rs - defines the core `CsvRecord` type and parses dates

The main module simply defines the logic for loading the configuration, establishing the database connection,
and reading the file/directory contents.

# Planned Improvements
For myself, there's not much left to do. For others, this tool could certainly be improved by supporting
other databases, be less restrictive on both CSV and database schemas, and support other currency and date formats. 

But in the near term, I do plan on implementing the following:

## Import Resumption
This feature would assume that a source spreadsheet is exported to the same target csv file multiple times, and
there is little use in re-importing the same rows over and over again. In this scenario, the importer would select
the largest `tx_id` for a given `account` in the destination table, and only attempt to insert rows read from the
csv file where tx_id exceeds the queried maximum. 

This feature is a *slight* optimization. Unless you're dealing with thousands and thousands of rows, this tool
is already pretty quick. 

## Automatic DB Schema Creation
I would like the ability to automatically create the destination schema in the target database
if it doesn't already exist. 

# Database Included
There are some utility scripts to create an instance of postgres 11 and run it locally.
These scripts will initialize a database using a docker volume with the name `financedb_data`,
but that can be overridden via command line arguments. 

Database setup is a two-step process:
* initialization: `./scripts/init-postgres.sh`
* running container: `./scripts/run-postgres.sh`

### Database initialization
The init-postgres.sh script initializes a new docker volume with the name `financedb_data`
(this name can be overridden via command line; look at the script), and creates the initial
database core files under that volume. The init script then chowns the private database
files under that volume to the current user. 

### Running the database
The database is executed via `./scripts/run-postgres.sh`, and it assumes the same volume as 
the init script. It uses `postgres` for each of the arguments (database name, user, password, etc),
and it runs on port 15432 rather than 5432. Feel free to change as necessary.