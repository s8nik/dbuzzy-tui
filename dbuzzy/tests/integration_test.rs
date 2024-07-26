use dbuzzy::db::queries::{DatabaseTree, TreeItem, TreeItemKind};

mod db;

#[tokio::test]
#[ignore]
async fn test_db_tree() -> anyhow::Result<()> {
    let (container, pool) = db::setup().await?;

    let connection = pool.acquire().await?;
    let tree = DatabaseTree::load(&connection).await?;

    let tree_list = tree.as_ref();

    assert_eq!(tree_list.len(), 3);

    assert_eq!(
        *tree_list[0],
        TreeItem {
            indent: 0,
            name: "postgres".to_owned(),
            kind: TreeItemKind::Database { collapsed: false }
        }
    );

    assert_eq!(
        *tree_list[1],
        TreeItem {
            indent: 4,
            name: "foo".to_owned(),
            kind: TreeItemKind::Schema {
                collapsed: false,
                database: tree_list[0].clone()
            },
        }
    );

    assert_eq!(
        *tree.as_ref()[2],
        TreeItem {
            indent: 8,
            name: "bar".to_owned(),
            kind: TreeItemKind::Table {
                schema: tree_list[1].clone(),
                database: tree_list[0].clone()
            }
        }
    );

    container.stop().await?;
    Ok(())
}
