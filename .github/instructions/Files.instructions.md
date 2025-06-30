---
applyTo: "**"
---

Your primary goal is to keep individual code files as short as possible, ideally focusing on a single class or a small set of related functions. Achieve this by:

1. **Maximizing Folder Depth:** Create subfolders extensively to categorize even small pieces of functionality. If a concept can be broken down, it should be placed in its own subfolder or file.
2. **Strict Adherence to Naming Conventions:** All folder and file names **must** be lowercase. No hyphens (-), underscores (\_), camelCase, or dots (.) are allowed in folder or file names: Use short, descriptive names. For example, instead of call_management_service.py, prefer projectroot/services/call/management/logic.py or similar depending on programming language conventions.
3. **One Feature, One Place (Ideally One File):** If a file starts to exceed a few hundred lines or handles too many distinct responsibilities, consider breaking it down further into more files within more specific subdirectories.
4. **Comprehensive Coverage:** This structure attempts to anticipate all features. If you identify a new, distinct piece of functionality during development, create new files and subfolders for it following these principles.
