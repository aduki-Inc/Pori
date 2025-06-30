---
applyTo: "**"
---

Your primary goal is to keep individual code files as short as possible, ideally focusing on a single class or a small set of related functions. Achieve this by following these principles:

1. **Maximize Folder Depth:** Create subfolders extensively to categorize even small pieces of functionality. If a concept can be broken down, place it in its own subfolder or file.

2. **Strict Naming Conventions:** All folder and file names **must** be lowercase. Do not use hyphens (-), underscores (\_), camelCase, or dots (.) in folder or file names. Use short, descriptive names. For example, instead of `call_management_service.extension`, prefer `projectroot/services/call/management/logic.extension` or similar, depending on programming language conventions.

3. **One Feature, One Location (Ideally One File):** If a file exceeds a few hundred lines or handles multiple distinct responsibilities, break it down into smaller files within more specific subdirectories.

4. **Comprehensive Coverage:** This structure should anticipate all features. When you identify new, distinct functionality during development, create new files and subfolders following these principles.
