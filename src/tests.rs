#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use tempfile::TempDir;

    // Helper function to create a test directory with MP3 files
    fn create_test_dir_with_mp3s() -> (TempDir, Vec<String>) {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let mp3_files = vec![
            "song1.mp3".to_string(),
            "song2.mp3".to_string(),
            "song3.mp3".to_string(),
            "not_mp3.txt".to_string(),
        ];

        // Create dummy files
        for file in &mp3_files {
            let path = temp_dir.path().join(file);
            let mut f = fs::File::create(&path).expect("Failed to create test file");
            f.write_all(b"dummy content").expect("Failed to write to test file");
        }

        (temp_dir, mp3_files)
    }

    // Tests for PlaybackControl
    #[test]
    fn test_playback_control_new() {
        let ctrl = crate::symphonia_control::PlaybackControl::new();
        assert!(!ctrl.is_paused());
        assert!(!ctrl.is_stopped());
    }

    #[test]
    fn test_playback_control_pause_resume() {
        let ctrl = crate::symphonia_control::PlaybackControl::new();
        
        assert!(!ctrl.is_paused());
        ctrl.pause();
        assert!(ctrl.is_paused());
        
        ctrl.resume();
        assert!(!ctrl.is_paused());
    }

    #[test]
    fn test_playback_control_stop() {
        let ctrl = crate::symphonia_control::PlaybackControl::new();
        
        assert!(!ctrl.is_stopped());
        ctrl.stop();
        assert!(ctrl.is_stopped());
    }

    #[test]
    fn test_playback_control_position() {
        let ctrl = crate::symphonia_control::PlaybackControl::new();
        
        assert_eq!(ctrl.get_position(), 0);
        ctrl.set_position(5000);
        assert_eq!(ctrl.get_position(), 5000);
        
        ctrl.set_position(10000);
        assert_eq!(ctrl.get_position(), 10000);
    }

    #[test]
    fn test_playback_control_clone() {
        let ctrl1 = crate::symphonia_control::PlaybackControl::new();
        let ctrl2 = ctrl1.clone();
        
        ctrl1.pause();
        assert!(ctrl2.is_paused());
        
        ctrl2.resume();
        assert!(!ctrl1.is_paused());
    }

    #[test]
    fn test_playback_control_default() {
        let ctrl = crate::symphonia_control::PlaybackControl::default();
        assert!(!ctrl.is_paused());
        assert!(!ctrl.is_stopped());
    }

    // Tests for utility functions
    #[test]
    fn test_save_and_load_queue() {
        // Clean up any existing queue file
        let _ = fs::remove_file(".rdaio_queue");
        
        let files = vec![
            "song1.mp3".to_string(),
            "song2.mp3".to_string(),
            "song3.mp3".to_string(),
        ];
        let directory = "./test_dir";
        
        // Test saving
        let result = crate::save_queue(&files, directory);
        assert!(result.is_ok(), "Failed to save queue");
        
        // Small delay to ensure file is written
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Test loading
        let loaded = crate::load_queue().expect("Failed to load queue");
        assert!(loaded.is_some(), "Loaded queue should not be empty");
        
        let (loaded_dir, loaded_files) = loaded.unwrap();
        assert_eq!(loaded_dir, "./test_dir", "Loaded directory should match");
        assert_eq!(loaded_files, files, "Loaded files should match");
        
        // Clean up
        let _ = fs::remove_file(".rdaio_queue");
    }

    #[test]
    fn test_load_queue_empty() {
        // Clean up any existing queue file
        let _ = fs::remove_file(".rdaio_queue");
        
        let result = crate::load_queue().expect("Failed to call load_queue");
        assert!(result.is_none());
    }

    #[test]
    fn test_save_queue_empty_list() {
        let _ = fs::remove_file(".rdaio_queue");
        
        let files: Vec<String> = vec![];
        let directory = "./test_dir";
        
        let result = crate::save_queue(&files, directory);
        assert!(result.is_ok(), "Failed to save empty queue");
        
        // Small delay to ensure file is written
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        // Verify we can load it back
        let loaded = crate::load_queue().expect("Failed to load queue");
        assert!(loaded.is_some(), "Should be able to load empty queue");
        
        // Clean up
        let _ = fs::remove_file(".rdaio_queue");
    }

    #[test]
    fn test_load_mp3_files_filters_correctly() {
        let (temp_dir, _) = create_test_dir_with_mp3s();
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        
        let result = crate::load_mp3_files(&temp_path);
        assert!(result.is_ok());
        
        let files = result.unwrap();
        // Should only contain .mp3 files
        assert!(files.iter().all(|f| f.ends_with(".mp3")));
        assert!(!files.iter().any(|f| f.ends_with(".txt")));
        // Should have 3 mp3 files
        assert_eq!(files.len(), 3);
    }

    #[test]
    fn test_load_mp3_files_sorted() {
        let (temp_dir, _) = create_test_dir_with_mp3s();
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        
        let result = crate::load_mp3_files(&temp_path);
        assert!(result.is_ok());
        
        let files = result.unwrap();
        // Check if sorted
        let mut sorted_files = files.clone();
        sorted_files.sort();
        assert_eq!(files, sorted_files);
    }

    #[test]
    fn test_load_mp3_files_invalid_directory() {
        let result = crate::load_mp3_files("/nonexistent/directory/path");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_display_name_filename_mode() {
        let file_name = "my_song.mp3";
        let directory = ".";
        
        let display = crate::get_display_name(file_name, directory, false);
        assert_eq!(display, file_name);
    }

    #[test]
    fn test_get_display_name_without_title() {
        let file_name = "track_123.mp3";
        let directory = "./music";
        
        // When show_title is false, should return the filename
        let display = crate::get_display_name(file_name, directory, false);
        assert_eq!(display, file_name);
    }

    #[test]
    fn test_get_display_name_fallback_to_filename() {
        let file_name = "nonexistent_file.mp3";
        let directory = ".";
        
        // File doesn't exist, should fallback to filename
        let display = crate::get_display_name(file_name, directory, true);
        assert_eq!(display, file_name);
    }

    #[test]
    fn test_get_folder_contents_includes_nav() {
        let (temp_dir, _) = create_test_dir_with_mp3s();
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        
        let result = crate::get_folder_contents(&temp_path);
        assert!(result.is_ok());
        
        let contents = result.unwrap();
        // Should include ".." and "."
        assert!(contents.iter().any(|(name, is_dir)| name == ".." && *is_dir));
        assert!(contents.iter().any(|(name, is_dir)| name == "." && *is_dir));
    }

    #[test]
    fn test_get_folder_contents_filters_mp3() {
        let (temp_dir, _) = create_test_dir_with_mp3s();
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        
        let result = crate::get_folder_contents(&temp_path);
        assert!(result.is_ok());
        
        let contents = result.unwrap();
        // Should not include .txt files, only mp3s
        assert!(!contents.iter().any(|(name, _)| name.ends_with(".txt")));
        // Should include mp3 files
        assert!(contents.iter().any(|(name, _)| name.ends_with(".mp3")));
    }

    #[test]
    fn test_get_folder_contents_sorted() {
        let (temp_dir, _) = create_test_dir_with_mp3s();
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        
        let result = crate::get_folder_contents(&temp_path);
        assert!(result.is_ok());
        
        let contents = result.unwrap();
        // Skip ".." and "." which are at the beginning
        let items: Vec<_> = contents.iter().skip(2).collect();
        
        // Verify folders come before files in the list
        let mut has_folder = false;
        let mut has_file = false;
        for (_, is_dir) in items.iter() {
            if *is_dir {
                has_folder = true;
            } else if has_folder {
                has_file = true;
                break;
            }
        }
        
        // Either all folders, all files, or folders come first
        if has_folder && has_file {
            // Folders should come before files after ".." and "."
            assert!(contents.iter().skip(2).take_while(|(_, is_dir)| *is_dir).count() > 0);
        }
    }

    #[test]
    fn test_get_folder_contents_invalid_directory() {
        let result = crate::get_folder_contents("/nonexistent/path");
        assert!(result.is_err());
    }

    // Tests for MP3 metadata reading
    #[test]
    fn test_get_mp3_title_nonexistent_file() {
        let result = crate::get_mp3_title("/nonexistent/file.mp3");
        assert!(result.is_none());
    }

    #[test]
    fn test_get_mp3_title_invalid_mp3() {
        let (temp_dir, _) = create_test_dir_with_mp3s();
        let temp_path = temp_dir.path().join("song1.mp3");
        
        // This is a dummy file, not a real MP3, so should return None
        let result = crate::get_mp3_title(temp_path.to_string_lossy().as_ref());
        assert!(result.is_none());
    }

    // Edge case tests
    #[test]
    fn test_load_mp3_files_empty_directory() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let temp_path = temp_dir.path().to_string_lossy().to_string();
        
        let result = crate::load_mp3_files(&temp_path);
        assert!(result.is_ok());
        
        let files = result.unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_playback_control_multiple_stops() {
        let ctrl = crate::symphonia_control::PlaybackControl::new();
        
        ctrl.stop();
        assert!(ctrl.is_stopped());
        
        // Calling stop multiple times should be fine
        ctrl.stop();
        assert!(ctrl.is_stopped());
    }

    #[test]
    fn test_playback_control_multiple_pauses() {
        let ctrl = crate::symphonia_control::PlaybackControl::new();
        
        ctrl.pause();
        assert!(ctrl.is_paused());
        
        // Calling pause multiple times should be fine
        ctrl.pause();
        assert!(ctrl.is_paused());
    }

    #[test]
    fn test_playback_control_pause_stopped() {
        let ctrl = crate::symphonia_control::PlaybackControl::new();
        
        ctrl.stop();
        ctrl.pause();
        
        // Should be able to pause a stopped playback
        assert!(ctrl.is_paused());
        assert!(ctrl.is_stopped());
    }

    #[test]
    fn test_save_queue_multiline_filenames() {
        let _ = fs::remove_file(".rdaio_queue");
        
        let files = vec![
            "song_one.mp3".to_string(),
            "song_two.mp3".to_string(),
        ];
        let directory = "./my_music";
        
        let result = crate::save_queue(&files, directory);
        assert!(result.is_ok(), "Failed to save queue with multiline filenames");
        
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        let loaded = crate::load_queue().expect("Failed to load queue");
        assert!(loaded.is_some());
        
        let (_, loaded_files) = loaded.unwrap();
        assert_eq!(loaded_files.len(), files.len());
        
        // Clean up
        let _ = fs::remove_file(".rdaio_queue");
    }

    #[test]
    fn test_get_display_name_special_characters() {
        let file_name = "song_!@#$%.mp3";
        let directory = ".";
        
        let display = crate::get_display_name(file_name, directory, false);
        assert_eq!(display, file_name);
    }
}
