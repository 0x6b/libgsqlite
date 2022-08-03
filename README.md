# libgsqlite

A [SQLite](https://www.sqlite.org) extension which loads a [Google Sheet](https://www.google.com/sheets/about/) as a virtual table.

https://user-images.githubusercontent.com/679719/182612984-3e1156c5-cc95-4450-bb20-2fd76f51aa3d.mp4

## Tested Platform

- [SQLite](https://www.sqlite.org) 3.39.2
- [Rust](https://www.rust-lang.org) 1.62.1-aarch64-apple-darwin
- macOS 12.5 (Monterey) on Apple M1 MAX

## Getting Started

### Setup Google Cloud

#### Create a Project

1. Log in to the [Google Cloud console](https://console.cloud.google.com/).
2. Go to the [**Manage resources**](https://console.cloud.google.com/cloud-resource-manager) page.
3. On the **Select organization** drop-down list at the top of the page, select the organization resource in which you want to create a project.
4. Click **Create Project**.
5. In the **New Project** window that appears, enter a project name, say `libgsqlite`, and select a billing account as applicable.
6. Enter the parent organization or folder resource in the **Location** box.
7. When you're finished entering new project details, click **Create**.

#### Enable Google Sheets API for the Project

1. Go to the [**API Library**](https://console.cloud.google.com/apis/library?project=_) page.
2. From the projects list, select the project you just created.
3. In the API Library, select **Google Sheets API**.
4. On the API page, click **Enable**.

#### Setup Google OAuth Consent Screen

1. Go to the [**OAuth consent screen**](https://console.cloud.google.com/apis/credentials/consent) page.
2. Select **Internal** as User Type, then click **Create**
3. Add required information like an app name (`libgsqlite`) and support email address.
4. Click **Save and Continue**.
5. Click **Add or Remove Scopes**.
6. On the dialog that appears, select the scope `.../auth/spreadsheets.readonly` (See all your Google Sheets spreadsheets) and click **Update**.
7. Click **Save and Continue**.
8. Click **Back to Dashboard**.

#### Create a Credential

1. Go to the [**Credentials**](https://console.cloud.google.com/apis/credentials) page.
2. Click **Create Credentials** → **OAuth Client ID**.
3. Select **Desktop app** as Application Type.
4. Type `libgsqlite` as Name.
5. Click **Download JSON** to save your **Client ID** and **Client Secret** locally.

### Create a Sample Spreadsheet

1. Go to [sheet.new](https://sheet.new) to create a new spreadsheet, then copy and paste following data.

| Employee Number | First Name | Last Name | Department |
|----------------:|------------|-----------|------------|
|               1 | Christine  | Haas      | A00        |
|               2 | Michael    | Thompson  | B01        |
|               3 | Sally      | Kwan      | C01        |
|               4 | John       | Beyer     | E01        |
|               5 | Irving     | Stern     | D11        |
|               6 | Eva        | Pulaski   | E01        |

2. Copy the URL of the spreadsheet.

### Query the Spreadsheet with SQLite

1. Setup required environment variables with the credential:
   ```shell
   $ export LIBGSQLITE_GOOGLE_CLIENT_ID=... # client_id property in the downloaded JSON
   $ export LIBGSQLITE_GOOGLE_CLIENT_SECRET=... # client_secret property
   ```
2. Launch SQLite:
   ```shell
   $ sqlite3
   ```
3. Load the extension:
   ```shell
   .load libgsqlite # or "gsqlite" on Windows
   ```
   If you get `Error: unknown command or invalid arguments: "load". Enter ".help" for help `, your SQLite is not capable for loading an extension. For macOS, install it with `brew install sqlite3`, and use it.
4. Create a virtual table for your spreadsheet by providing `ID` (url of the spreadsheet), `SHEET` (sheet name), and `RANGE` for module arguments. All three arguments are mandatory. You'll be navigated to Google OAuth consent screen to get a secret to access the spreadsheet. You can create multiple virtual tables from different spreadsheets.
   ```sql
   CREATE VIRTUAL TABLE employees USING gsqlite(
       ID 'https://docs.google.com/spreadsheets/d/...', -- your spreadsheet URL
       SHEET 'Sheet1', -- name of the sheet
       RANGE 'A2:D7' -- range to fetch
   );
   ```
5. Go back to your terminal, and run a query as usual:
   ```sql
   .mode column
   .headers on
   SELECT * FROM employees;
   SELECT * FROM employees WHERE D LIKE 'E%';
   ```

# Contributing

Please read [CONTRIBUTING](CONTRIBUTING.md) for more detail.

# Acknowledgements

An article, [Extending SQLite with Rust to support Excel files as virtual tables | Sergey Khabibullin](https://sergey.khabibullin.com/sqlite-extensions-in-rust/), and its companion repository [x2bool/xlite](https://github.com/x2bool/xlite), for great write up and inspiration.

# Limitations

- The extension will load the spreadsheet only once while creating a virtual table. If you want to pick up recent changes, drop the table and create it again.
- `INSERT`, `UPDATE` and `DELETE` statements won't be implemented. Welcome PRs.

# Security

The extension is intended for use in personal, not-shared, environment. The Google Cloud secret will be cached for 59 minutes under the temporary directory (See [`std::env::temp_dir`](https://doc.rust-lang.org/std/env/fn.temp_dir.html)) with fixed name `access_token.json` for your convenience. Note that, as described at the doc, creating a file or directory with a fixed or predictable name may result in “insecure temporary file” security vulnerability.

# Privacy

The extension never send your data to any server.

# License

This extension is released under the MIT License. See [LICENSE](LICENSE) for details.

# References

- [Quickstart: Manage your Google Cloud resources](https://cloud.google.com/resource-manager/docs/manage-google-cloud-resources#create_a_project_resource)
- [Getting started | Cloud APIs](https://cloud.google.com/apis/docs/getting-started)
- [Setting up OAuth 2.0](https://support.google.com/cloud/answer/6158849?hl=en&ref_topic=3473162)
