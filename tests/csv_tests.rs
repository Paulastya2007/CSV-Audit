use std::fs::{self, File};
use software_rust::csv::helpdir::find_csv_files;
use software_rust::csv::backend::inspect_csv;

#[test]
fn test_find_csv_files_integration() {
    let mut root = std::env::temp_dir();
    root.push("test_find_csv_files_integration_dir");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    // 1. Create a CSV file in root
    let root_csv = root.join("file1.csv");
    File::create(&root_csv).unwrap();

    // 2. Create a non-CSV file in root
    let root_txt = root.join("file2.txt");
    File::create(&root_txt).unwrap();

    // 3. Create a subdirectory and a CSV file inside it
    let sub_dir = root.join("subdir");
    fs::create_dir(&sub_dir).unwrap();
    let sub_csv = sub_dir.join("file3.csv");
    File::create(&sub_csv).unwrap();

    // 4. Create a deeply nested directory with a CSV
    let deep_dir = sub_dir.join("deep");
    fs::create_dir(&deep_dir).unwrap();
    let deep_csv = deep_dir.join("file4.csv");
    File::create(&deep_csv).unwrap();

    let found = find_csv_files(&root);

    // We expect only file1.csv and file3.csv (one level deep)
    // file4.csv should be excluded because it's two levels deep
    let expected = vec![root_csv, sub_csv];
    assert_eq!(found, expected);

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn test_inspect_csv_integration() {
    let mut root = std::env::temp_dir();
    root.push("test_inspect_csv_integration_dir");
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();

    let csv_file = root.join("passwords.csv");
    fs::write(
        &csv_file,
        "username,password,service\nadmin,admin123,router\nuser1,Password!1,email\n",
    )
    .unwrap();

    let info = inspect_csv(&csv_file).unwrap();
    assert_eq!(info.row_count, 2);
    assert!(info.size_bytes > 0);

    let _ = fs::remove_dir_all(&root);
}
