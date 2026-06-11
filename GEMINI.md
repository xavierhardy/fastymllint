# AI Instructions for fastymllint

To ensure consistency and quality in this project, all AI agents MUST follow these instructions:

1.  **Read Documentation First**: Always read `README.md` and any other high-level documentation in the `docs/` folder before reading code.
2.  **Use Task Files**:
    *   Consult `TASKS.md` before starting any work to understand what needs to be done.
    *   Update `TASKS.md` by removing or marking tasks as you work on them.
    *   Update `docs/DONE_TASKS.md` immediately after completing a task. Use the format: `- YYYY-MM-DD: Task description`.
3.  **Adhere to Conventions**: Strictly follow the established Rust coding style and project architecture.
4.  **Test-Driven Development**: Always add unit tests for new features or bug fixes.
5.  **Stay Focused**: Do not take significant actions beyond the scope of the current task without confirmation.
6.  **Update Documentation**: Ensure all relevant documentation (README, architecture docs, rule docs) is updated for every task you perform.

## Mandatory Verification (Every Task)

You MUST run these commands for EVERY task before completion:
- **Lint & Format**: `./lint-format.sh`
- **Test**: `./test.sh`
- **Build**: `./build.sh`

## Workflow
1. Read `README.md`.
2. Read `TASKS.md`.
3. Perform the task.
4. **Update Documentation**: Ensure all relevant docs are up-to-date.
5. **Verify**: Run Lint, Test, and Build (MUST pass).
6. Update `TASKS.md` and `docs/DONE_TASKS.md`.
