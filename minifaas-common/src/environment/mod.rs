mod environment;
mod environments;

pub use environment::Environment;
pub use environments::Environments;

#[cfg(test)]
mod tests {
    use super::*;

    use async_std::fs::read;
    use minifaas_test::get_empty_tmp_dir;
    use uuid::Uuid;

    #[async_std::test]
    async fn test_env_add_file_readable() {
        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let f_name = "hello";
        let env_path = root_dir.join(Uuid::new_v4().to_string());
        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();

        assert!(e.add_file(f_name).is_ok());
        assert_eq!(read(env_path.join(f_name)).await.unwrap(), Vec::<u8>::new()); // the file is empty
        assert!(std::fs::remove_dir_all(root_dir).is_ok());
    }

    #[async_std::test]
    async fn test_env_add_file_duplicate() {
        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let f_name = "hello";
        let env_path = root_dir.join(Uuid::new_v4().to_string());

        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();

        assert!(e.add_file(f_name).is_ok());
        assert!(e.add_file(f_name).is_ok());
        assert!(std::fs::remove_dir_all(root_dir).is_ok());
    }

    #[async_std::test]
    async fn test_env_add_file_async_readable() {
        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let f_name = "hello";
        let env_path = root_dir.join(Uuid::new_v4().to_string());
        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();

        assert!(e.add_file_async(f_name).await.is_ok());
        assert_eq!(read(env_path.join(f_name)).await.unwrap(), Vec::<u8>::new()); // the file is empty
        assert!(std::fs::remove_dir_all(root_dir).is_ok());
    }

    #[async_std::test]
    async fn test_env_add_file_async_duplicate() {
        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let f_name = "hello";
        let env_path = root_dir.join(Uuid::new_v4().to_string());

        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();

        assert!(e.add_file_async(f_name).await.is_ok());
        assert!(e.add_file_async(f_name).await.is_ok());
        assert!(std::fs::remove_dir_all(root_dir).is_ok());
    }

    #[async_std::test]
    async fn test_env_add_dir_readable() {
        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let f_name = "hello";
        let env_path = root_dir.join(Uuid::new_v4().to_string());
        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();

        assert!(e.add_dir(f_name).await.is_ok());

        let metadata = async_std::fs::metadata(env_path.join(f_name))
            .await
            .unwrap();

        assert!(metadata.file_type().is_dir());
        assert!(std::fs::remove_dir_all(root_dir).is_ok());
    }

    #[async_std::test]
    async fn test_env_add_dir_duplicate() {
        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let f_name = "hello";
        let env_path = root_dir.join(Uuid::new_v4().to_string());

        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();

        assert!(e.add_dir(f_name).await.is_ok());
        assert!(e.add_dir(f_name).await.is_ok());
        assert!(std::fs::remove_dir_all(root_dir).is_ok());
    }

    #[async_std::test]
    async fn test_env_has_dir_valid() {
        let f_name = "hello";

        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let env_path = root_dir.join(Uuid::new_v4().to_string());
        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();
        async_std::fs::create_dir(env_path.join(f_name))
            .await
            .unwrap();
        assert!(e.has_dir(f_name).await);
        assert!(!e.has_dir("other_dir").await);
    }

    #[async_std::test]
    async fn test_env_has_dir_no_files() {
        let f_name = "a-file";

        let root_dir = get_empty_tmp_dir();
        let expected_id = Uuid::new_v4();
        let env_path = root_dir.join(Uuid::new_v4().to_string());
        let e = Environment::create_with_id(env_path.clone(), expected_id)
            .await
            .unwrap();
        async_std::fs::write(env_path.join(f_name), b"content")
            .await
            .unwrap();
        assert!(!e.has_dir(f_name).await);
    }
}
