use wikipedia2jpdb::category::Category;

#[test]
fn test_sub() {
    tokio_test::block_on(Category::subcategories("Category:天文学"));
}

#[test]
fn test_supercategories() {
    tokio_test::block_on(Category::parent_categories("Category:天文学", true));
}
