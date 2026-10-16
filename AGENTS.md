# Repository Guidelines

## Project Structure & Module Organization

This workspace currently has no application code, tests, assets, or build configuration. Establish a clear directory layout when adding the first implementation. Prefer `src/` for source code, `tests/` for automated tests, and `assets/` for static resources when applicable. Group related functionality together and document any different layout in `README.md`.

## Build, Test, and Development Commands

No build, test, or local development commands are configured yet. When introducing tooling, provide reproducible commands in the appropriate package manifest or build file, and explain them in `README.md`. Include dependency installation, local execution, testing, and production builds where relevant. Do not document commands as available until their configuration exists.

## Coding Style & Naming Conventions

No language, formatter, or linter has been selected. Adopt the standard conventions of the chosen language and configure formatting and linting alongside the first source files. Keep indentation consistent within each file. Use descriptive names, follow ecosystem conventions for filenames, and avoid mixing unrelated changes in one contribution.

## Testing Guidelines

No testing framework or coverage threshold is configured. Add suitable automated tests with new behavior and regression tests for bug fixes. Use descriptive test names that identify the behavior and expected outcome. Document test discovery conventions and the exact command needed to run the suite once a framework is selected.

## Commit & Pull Request Guidelines

No Git history is available to establish existing commit conventions. Use concise, imperative commit subjects, such as `Add initial project structure`. Keep commits focused. Pull requests should explain the change, link relevant issues, and report checks performed or limitations. Include screenshots for visible interface changes.

## Security & Configuration

Keep credentials and local configuration out of version control. Provide sanitized configuration examples and ignore generated files when introducing tooling.
