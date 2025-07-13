---
applyTo: "**"
---

1. **Verify Project Structure:** Examine existing architecture before changing code. Follow established patterns.

2. **Ask Questions:** When uncertain, ask for clarification. Never invent requirements or dependencies.

3. **Keep Files Small:** Focus on single responsibility per file - one class or related function set.

4. **Create Deep Folder Hierarchies:** Use multiple nested folders to organize functionality.

5. **Naming Rules:**

- All folders and files: lowercase only
- No hyphens, underscores, camelCase or dots in names
- Example: use `services/call/management/logic.ext` not `call_management_service.ext`

6. **Split Large Files:** Files exceeding 200-300 lines or with multiple responsibilities must be divided into smaller files in specific subdirectories.

7. **Plan Ahead:** Create folder structures anticipating all functionality, including future features.
