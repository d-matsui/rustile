## [0.8.3](https://github.com/d-matsui/rustile/compare/v0.8.2...v0.8.3) (2025-09-12)

### 🐛 Bug Fixes

* change default_display from :10 to :0 for production use ([3b9697d](https://github.com/d-matsui/rustile/commit/3b9697d5701ebcde678f221bbf40d9385dd9be21))

### 📖 Documentation

* reorganize ROADMAP with 5 clear functional categories ([18d5bee](https://github.com/d-matsui/rustile/commit/18d5bee8fb071cb33cf8d4beca932fb240b93a1a))

## [0.8.2](https://github.com/d-matsui/rustile/compare/v0.8.1...v0.8.2) (2025-09-10)

### 📖 Documentation

* update ROADMAP.md to mark architecture refactoring as completed ([9bf9f43](https://github.com/d-matsui/rustile/commit/9bf9f436bfcf1630c39c8bed067fcf6f21f13f88)), closes [#43](https://github.com/d-matsui/rustile/issues/43)

### 💎 Style

* apply cargo fmt to ensure consistent formatting ([912136f](https://github.com/d-matsui/rustile/commit/912136fa38f3ccdebc320d6a6ba15047bc931243))

### ♻️ Refactor

* separate BSP tree logic from screen geometry calculations ([747e242](https://github.com/d-matsui/rustile/commit/747e24286844c2a593c14d99409be50fdc76a72d))

## [0.8.1](https://github.com/d-matsui/rustile/compare/v0.8.0...v0.8.1) (2025-09-10)

### 🔧 CI/CD

* **deps:** bump actions/setup-node from 4 to 5 ([0333567](https://github.com/d-matsui/rustile/commit/0333567ce75dd08241b483a6016cca177e9b7e92))

## [0.8.0](https://github.com/d-matsui/rustile/compare/v0.7.15...v0.8.0) (2025-09-02)

### 🚀 Features

* implement zoom to parent feature ([028aa93](https://github.com/d-matsui/rustile/commit/028aa9389d0cd9375b17b1ee3a7f0c2d73c02d05))
* implement zoom to parent feature ([4eb4d7f](https://github.com/d-matsui/rustile/commit/4eb4d7f958da33e25add3c712bf3ff41e7e1308d))

## [0.7.15](https://github.com/d-matsui/rustile/compare/v0.7.14...v0.7.15) (2025-08-26)

### 🔧 CI/CD

* **deps:** bump actions/checkout from 4 to 5 ([b0d0cb6](https://github.com/d-matsui/rustile/commit/b0d0cb674eeae5b797b0057d7dc1a7ed4417c0f0))

## [0.7.14](https://github.com/d-matsui/rustile/compare/v0.7.13...v0.7.14) (2025-08-13)

### 🐛 Bug Fixes

* improve CI stability and resolve dependabot issues ([2d0f02d](https://github.com/d-matsui/rustile/commit/2d0f02d19382dcdee045215411c940886f34ba2f))
* resolve clippy collapsible_if warnings in Rust 1.89 ([286edd7](https://github.com/d-matsui/rustile/commit/286edd760a5a4794f4cf5d85e496ee5dc59bff15))
* resolve unused debug imports in release builds ([e731c98](https://github.com/d-matsui/rustile/commit/e731c9814f7e15d2271e3131a69931d968a96892))

### 💎 Style

* fix import order after rebase ([1d7435f](https://github.com/d-matsui/rustile/commit/1d7435f8baa0115f7ba9e4e7947bd9865a8730ee))
* fix let-chains formatting for Rust 1.89 ([e3c37e4](https://github.com/d-matsui/rustile/commit/e3c37e44a8a6918f31b4b53b421aaebdcc0bf60f))

## [0.7.13](https://github.com/d-matsui/rustile/compare/v0.7.12...v0.7.13) (2025-08-07)

### ♻️ Refactor

* remove AltGr support from codebase ([46483b3](https://github.com/d-matsui/rustile/commit/46483b3779d40d918194e0c33345cfe897b1df83))
* unify KeyParser and KeyboardManager into ShortcutManager ([8b92c9d](https://github.com/d-matsui/rustile/commit/8b92c9d5808dd1bc489a1caa7e0669c37b04bec1))

## [0.7.12](https://github.com/d-matsui/rustile/compare/v0.7.11...v0.7.12) (2025-08-06)

### 📖 Documentation

* clarify X11 keyboard mapping concepts and add ADR-007 ([b14b960](https://github.com/d-matsui/rustile/commit/b14b960357eb57f24f7de71b4400e8fcffd891a9))
* update roadmap priorities and fix ADR naming convention ([e714804](https://github.com/d-matsui/rustile/commit/e714804d05e4ec01a56dd172efa0755d5984bb24))

### ♻️ Refactor

* improve keyboard terminology and add modifier documentation ([c8612e7](https://github.com/d-matsui/rustile/commit/c8612e7f689c742f7b1f7a805cf09faba46866cb))

## [0.7.11](https://github.com/d-matsui/rustile/compare/v0.7.10...v0.7.11) (2025-08-05)

### 🐛 Bug Fixes

* restore ConfigureRequest handler to resolve xterm launch performance regression ([f26cedd](https://github.com/d-matsui/rustile/commit/f26ceddbf6c7d33ff61eb7e4b044dccdb263cb36))
* restore ConfigureRequest handler to resolve xterm launch performance regression ([ab43756](https://github.com/d-matsui/rustile/commit/ab43756226031ca3df19c78a454a55a42ba7ec17))

## [0.7.10](https://github.com/d-matsui/rustile/compare/v0.7.9...v0.7.10) (2025-08-05)

### ♻️ Refactor

* remove unused event handlers for cleaner architecture ([33f4971](https://github.com/d-matsui/rustile/commit/33f497140d7bf96c856d80c3b0f259b59bebad1b))

## [0.7.9](https://github.com/d-matsui/rustile/compare/v0.7.8...v0.7.9) (2025-08-05)

### 📖 Documentation

* add ADR-005 and update CLAUDE.md with code comment standard ([eb6d317](https://github.com/d-matsui/rustile/commit/eb6d3179604eb6498e4a3f99eb9782ca9a72da1e))
* reorganize CLAUDE.md for improved readability and structure ([177b4d0](https://github.com/d-matsui/rustile/commit/177b4d04b5678f1f3a7e85fe65debad762ac3b12))

### ♻️ Refactor

* implement concise code comment standard ([6d93b69](https://github.com/d-matsui/rustile/commit/6d93b69f0adc93493e726f4e0b8e133e28e55567))

## [0.7.8](https://github.com/d-matsui/rustile/compare/v0.7.7...v0.7.8) (2025-08-05)

### ♻️ Refactor

* implement unified rendering API with single apply_state method ([3fe08cf](https://github.com/d-matsui/rustile/commit/3fe08cf53500c826c20fb78f7a60db512276b0a3))

## [0.7.7](https://github.com/d-matsui/rustile/compare/v0.7.6...v0.7.7) (2025-08-04)

### 💎 Style

* apply cargo fmt formatting to debug logging statements ([2367d50](https://github.com/d-matsui/rustile/commit/2367d50fd56efe2ef97300cc9b9d57cbad5262e4))

### ♻️ Refactor

* enhance logging with window names and detailed event information ([c1eae27](https://github.com/d-matsui/rustile/commit/c1eae271879584c06d71c09fa1ed6904d4a5b27a))
* revert window name resolution complexity, add to roadmap ([ceb1485](https://github.com/d-matsui/rustile/commit/ceb1485bc65126d5083e45fc52a56900991199e9))
* standardize logging with simplified 3-level approach ([044f95b](https://github.com/d-matsui/rustile/commit/044f95b1e6f855f5fd9aae7ba5edcaceb915da7c))

## [0.7.6](https://github.com/d-matsui/rustile/compare/v0.7.5...v0.7.6) (2025-08-04)

### 🐛 Bug Fixes

* clean up duplicate content in CHANGELOG.md ([f81cc27](https://github.com/d-matsui/rustile/commit/f81cc272858be93c1df04b0b2fc379808d18158c))

### 📖 Documentation

* add focused implementation documentation ([9e84763](https://github.com/d-matsui/rustile/commit/9e847638728f3b7d4e77f1505e7464589b3a1605))
* update README and CLAUDE.md for new documentation structure ([9625af5](https://github.com/d-matsui/rustile/commit/9625af54ce86cb130b352c56db319de0ac1ace65))

## [0.7.5](https://github.com/d-matsui/rustile/compare/v0.7.4...v0.7.5) (2025-08-04)

### 🐛 Bug Fixes

* apply cargo fmt to pass check.sh ([310e1cb](https://github.com/d-matsui/rustile/commit/310e1cb299a43edf7e4c455d76a84371d4d195cb))
* apply formatting and remove dead code per CLAUDE.md requirements ([41bd1c1](https://github.com/d-matsui/rustile/commit/41bd1c187c0f5d616c7ff571324fe44d009746b8))

### 📖 Documentation

* add ADR-003 for SRP refactoring to three-module architecture ([c0150dd](https://github.com/d-matsui/rustile/commit/c0150ddae69f28ce60087f9eec61fd06f6ee2566))

### ♻️ Refactor

* complete Phase 2 SRP refactoring with window_operations extraction ([6dc07ad](https://github.com/d-matsui/rustile/commit/6dc07ad3fdc9455e108ba7a75920a4fcfed6ff26))
* complete SRP refactoring with 3-module architecture ([e514d36](https://github.com/d-matsui/rustile/commit/e514d367d7d40b962bc1bc4de1000f2fa0e533b1))
* extract event_handler.rs module (Phase 1) ([f8fed26](https://github.com/d-matsui/rustile/commit/f8fed267523e21290c0bafc67796b2e75ee84e74))
* Phase 4 - Extract WindowState module ([0892aba](https://github.com/d-matsui/rustile/commit/0892aba10e218219a3ad982b3cdfb285b48feb2e))
* Phase 5 - Extract WindowRenderer module ([c3d9102](https://github.com/d-matsui/rustile/commit/c3d910212e1fce36f718230122018125b158e602))

## [0.7.4](https://github.com/d-matsui/rustile/compare/v0.7.3...v0.7.4) (2025-08-01)

### 📖 Documentation

* add forbidden Rust attributes section to CLAUDE.md ([0ea5239](https://github.com/d-matsui/rustile/commit/0ea5239343600d93e14e755a9d57b834c2d89ab0))

### ♻️ Refactor

* add get_first_window() helper to eliminate magic 0 value ([d1ea47f](https://github.com/d-matsui/rustile/commit/d1ea47f551e13855dfc2039a366d043950981479))
* remove redundant window_stack field and MRU logic ([3abd679](https://github.com/d-matsui/rustile/commit/3abd679aadd8fe43e3377877c2e73984b0bc8de9))

## [0.7.3](https://github.com/d-matsui/rustile/compare/v0.7.2...v0.7.3) (2025-07-31)

### ♻️ Refactor

* add focus state color logic helper method ([6ca40ff](https://github.com/d-matsui/rustile/commit/6ca40ff829691dbb8c8ed0622d61c44a260214e2))
* add window border configuration helper method ([a8dfc68](https://github.com/d-matsui/rustile/commit/a8dfc6861dbc29acfb8f6b375ae6186a531bdacf))
* bundle layout parameters to reduce duplication ([5de4934](https://github.com/d-matsui/rustile/commit/5de4934715ca7a7f9f24648d85719cf8f933d5fc))

## [0.7.2](https://github.com/d-matsui/rustile/compare/v0.7.1...v0.7.2) (2025-07-31)

### 💎 Style

* apply cargo fmt formatting ([d90f40d](https://github.com/d-matsui/rustile/commit/d90f40d39b20aaf9584d75a5969a93903a2a7c02))

### ♻️ Refactor

* remove all dead code and unused functionality ([707547a](https://github.com/d-matsui/rustile/commit/707547aad2e9b908701ee32a5d200f2bfb8884b7))
* simplify file structure to flat organization ([3ca39f6](https://github.com/d-matsui/rustile/commit/3ca39f69aa72a2773cb08fc257da36326377250c))

## [0.7.1](https://github.com/d-matsui/rustile/compare/v0.7.0...v0.7.1) (2025-07-30)

### 🐛 Bug Fixes

* correct rotation example in ADR-001 ([608a98a](https://github.com/d-matsui/rustile/commit/608a98a6fae5282c4451d221a50f87f27b07aef5))

### 📖 Documentation

* add ADR-002 for single source of truth architecture ([94c4f38](https://github.com/d-matsui/rustile/commit/94c4f382460514a23303de378e93e3d2ee86e30c))
* add concise example to ADR-001 for clarity ([11dc392](https://github.com/d-matsui/rustile/commit/11dc39242b935fffbb17f631a1e1d1dcecd874ef))
* improve ADR-001 to follow best practices ([bece083](https://github.com/d-matsui/rustile/commit/bece08341082d06488764964c2717af2c2b2ff19))
* improve ADR-002 to follow best practices ([518bcf6](https://github.com/d-matsui/rustile/commit/518bcf6b3c0276896164885fbc717b665a655ffd))

### ♻️ Refactor

* enhance BspTree API with window management methods ([8991a62](https://github.com/d-matsui/rustile/commit/8991a625bc3dab0123216efe0d97cfa5f07d3c97))
* remove master window concept and swap_with_master feature ([ceefc6f](https://github.com/d-matsui/rustile/commit/ceefc6fa914aaabbeb81c96adf44ac042526bf58))
* remove windows vector and achieve single source of truth ([adc0b1b](https://github.com/d-matsui/rustile/commit/adc0b1bee0dc3071b2de279b88e5d4d4ad453121))
* use intuitive SplitDirection terminology ([313d881](https://github.com/d-matsui/rustile/commit/313d881ec6b7323556fafa7421f71cf3e4baf3b2))

## [0.7.0](https://github.com/d-matsui/rustile/compare/v0.6.1...v0.7.0) (2025-07-30)

### 🚀 Features

* implement window rotation functionality with BSP tree preservation ([0525642](https://github.com/d-matsui/rustile/commit/05256423acbc2aaa1d5510c09c81ee39613ddff8))
* update keyboard shortcuts and roadmap for rotate feature ([7a92f60](https://github.com/d-matsui/rustile/commit/7a92f605ed4c337d3d0c39a92f5f8f7fa1e219f0))

## [0.6.1](https://github.com/d-matsui/rustile/compare/v0.6.0...v0.6.1) (2025-07-29)

### 📖 Documentation

* add comprehensive architecture explanation to technical guide ([48def8c](https://github.com/d-matsui/rustile/commit/48def8cd15ff89cba97001fcdee860f1093fc85f))
* update documentation to reflect BSP-only architecture ([8800c4b](https://github.com/d-matsui/rustile/commit/8800c4bba1678b07f34d277f73d5365209353397))

### ♻️ Refactor

* remove LayoutManager abstraction ([3dad536](https://github.com/d-matsui/rustile/commit/3dad536bcef42a129dd7d60b7ce8e525c60c0807))
* separate X11 operations from layout calculations ([cd914d4](https://github.com/d-matsui/rustile/commit/cd914d4efbb52fc6829f9d5ba644bf2d69d20773))
* simplify to BSP-only layout algorithm ([006e430](https://github.com/d-matsui/rustile/commit/006e430fbb3013aa679c52e7f58b4a600aa142f4))

## [0.6.0](https://github.com/d-matsui/rustile/compare/v0.5.1...v0.6.0) (2025-07-29)

### 🚀 Features

* change fullscreen shortcut to AltGr+f (Right Alt) ([b007c87](https://github.com/d-matsui/rustile/commit/b007c87f5e853bbb846ad04f0496b085a54b1038))
* implement complete fullscreen toggle functionality ([f3369e3](https://github.com/d-matsui/rustile/commit/f3369e34b501c8e52ce1fe6abb6474fc054cc0de))
* implement fullscreen toggle feature ([4d9b1b6](https://github.com/d-matsui/rustile/commit/4d9b1b63baf87c063e92f394cf7cede473beb635))

### 🐛 Bug Fixes

* fullscreen mode now properly handles focus changes ([50ecc00](https://github.com/d-matsui/rustile/commit/50ecc00e2b552436905c72fef211ef774754f54e))

### 🧪 Tests

* add comprehensive fullscreen state transition tests ([348a8c5](https://github.com/d-matsui/rustile/commit/348a8c585758a6aac9e0c5bebb6db3fab22e687f))

## [0.5.1](https://github.com/d-matsui/rustile/compare/v0.5.0...v0.5.1) (2025-07-28)

### 🐛 Bug Fixes

* resolve PR review issues with broken references and commands ([8e739af](https://github.com/d-matsui/rustile/commit/8e739af8f48df96876c2029c25c017de1b1ff083))

### 📖 Documentation

* comprehensive README update with user-focused content ([9da0ab4](https://github.com/d-matsui/rustile/commit/9da0ab44bddb6142c1a5263d64af4559963ceb2b))
* fix FIXME comments and improve figure formatting for GitHub ([bc3e823](https://github.com/d-matsui/rustile/commit/bc3e823e5e15fc108da359d79d0714bf09c5d89b))
* remove FIXME comment from beginner guide ([92b814f](https://github.com/d-matsui/rustile/commit/92b814f0378c28441e2796b2811abe415747e109))
* restructure documentation with simplified beginner and technical guides ([4fa6e1a](https://github.com/d-matsui/rustile/commit/4fa6e1a4a77fcd16877ac92cc4cd7544f219a9e6))

### ♻️ Refactor

* simplify test scripts following KISS principle ([04ee3f3](https://github.com/d-matsui/rustile/commit/04ee3f342b9ec1f770cdf34bc0f22aac7d95563d))

## [0.5.0](https://github.com/d-matsui/rustile/compare/v0.4.0...v0.5.0) (2025-07-24)

### 🚀 Features

* implement window swapping functionality with Shift+Alt+j/k shortcuts ([115774b](https://github.com/d-matsui/rustile/commit/115774bd018ecf57bd1c59abdf1ba48648863995))

## [0.4.0](https://github.com/d-matsui/rustile/compare/v0.3.4...v0.4.0) (2025-07-24)

### 🚀 Features

* implement destroy/close window functionality ([0dee404](https://github.com/d-matsui/rustile/commit/0dee40428c9ef384294d1a4a1a7b38c1a5ab2b0d))

### 📖 Documentation

* comprehensive documentation update with GitHub-friendly ASCII art ([b7cbfa4](https://github.com/d-matsui/rustile/commit/b7cbfa42d1fc100f93c508a4e1ff2aac46577918))

## [0.3.4](https://github.com/d-matsui/rustile/compare/v0.3.3...v0.3.4) (2025-07-23)

### 🐛 Bug Fixes

* resolve cargo fmt formatting issues for CI compliance ([72bb8c2](https://github.com/d-matsui/rustile/commit/72bb8c2f7f1f79e828d1cb856a1a71520e777d89))
* resolve compiler warnings for conditional debug imports ([ddd4dc2](https://github.com/d-matsui/rustile/commit/ddd4dc220b670048b191dd295571669416319406))
* swap_with_master now reapplies layout to update window positions ([d9f088e](https://github.com/d-matsui/rustile/commit/d9f088e148101742a8853ad34b05c927cdf3a892))
* update default display from :1 to :10 for test environment compatibility ([7c9df1f](https://github.com/d-matsui/rustile/commit/7c9df1faca598f66193c83e1df683b2452f2d645))

### 📖 Documentation

* add comprehensive educational documentation with visual diagrams ([19b785e](https://github.com/d-matsui/rustile/commit/19b785ead55d4e412192ddd4fa42a4efbc6c0ae4))
* add destroy window to roadmap basic features ([af33c7d](https://github.com/d-matsui/rustile/commit/af33c7ded51d38f6599c5d71c25eeaaf26e1a9b0))
* enforce zero-warning builds and fix code formatting ([b8a2aff](https://github.com/d-matsui/rustile/commit/b8a2aff117c99b36fb6e9519d341055e968a8466))
* restore yabai reference in README acknowledgments ([942226c](https://github.com/d-matsui/rustile/commit/942226ccfecebe80b4f3816c731f1bbd4ecd8ec5))
* update ROADMAP.md with refined development priorities ([db3b4a1](https://github.com/d-matsui/rustile/commit/db3b4a13d4ed411b2e67882d8cf8ec4116574bec))

### ♻️ Refactor

* complete layout.rs modularization with improved code organization ([c854661](https://github.com/d-matsui/rustile/commit/c85466139b04c467b7eb4e130f44a052533998ce))
* extract constants and introduce parameter structs for better maintainability ([40b3b2b](https://github.com/d-matsui/rustile/commit/40b3b2b6aff3ff19bf213399819fbbe1184307cc))
* implement layout trait and improve config validation system ([f3f6964](https://github.com/d-matsui/rustile/commit/f3f6964635a48212d714e2f482d780245908a627))
* split layout.rs into modules and remove yabai references ([565b6d7](https://github.com/d-matsui/rustile/commit/565b6d7b76787a0e6b69264320036f16962a762f))
* split window_manager.rs into focused modules for better organization ([e9a253a](https://github.com/d-matsui/rustile/commit/e9a253adbd5195b57a013fc4289c14a83be9f226))

### 🧪 Tests

* add comprehensive window manager tests for core business logic ([158915b](https://github.com/d-matsui/rustile/commit/158915b6b73cec3e6b37aca16aa3be6d55051c0e))
* enhance test script to open 4 diverse X11 applications ([4826d4f](https://github.com/d-matsui/rustile/commit/4826d4f80da4c6a5015162b1b4aea26860f585e4))

## [0.3.3](https://github.com/d-matsui/rustile/compare/v0.3.2...v0.3.3) (2025-07-23)

### 🐛 Bug Fixes

* prevent docs commits from triggering releases ([c88ab8c](https://github.com/d-matsui/rustile/commit/c88ab8c08c2601887d9263982b2e231d463a5bed))

## [0.3.2](https://github.com/d-matsui/rustile/compare/v0.3.1...v0.3.2) (2025-07-23)

### 📖 Documentation

* improve roadmap structure and acknowledgments ([5bd3022](https://github.com/d-matsui/rustile/commit/5bd30226c549afc34f06603e8f3b9a0564cdefe0))

## [0.3.1](https://github.com/d-matsui/rustile/compare/v0.3.0...v0.3.1) (2025-07-23)

### 🐛 Bug Fixes

* use PAT for semantic-release to bypass branch protection ([a333807](https://github.com/d-matsui/rustile/commit/a33380709ed8186b997a25e116faec89766721cf))

### 📖 Documentation

* reorganize documentation structure and simplify README ([d658f52](https://github.com/d-matsui/rustile/commit/d658f52e02ca68bb6e9d91391714f8892c6ce839))

## [0.3.0](https://github.com/d-matsui/rustile/compare/v0.2.0...v0.3.0) (2025-07-23)

### ⚠ BREAKING CHANGES

* Consolidated CI jobs and Claude workflows for better performance

**CI Optimizations:**
- Merge test, format, clippy, docs into single 'test-and-quality' job
- Run security audit in parallel instead of sequentially
- Add conditional release builds (only for main/develop branches)
- Improve Rust caching with versioned keys
- Optimize system dependency installation

**Claude Workflow Consolidation:**
- Merge claude.yml and claude-code-review.yml into single workflow
- Add smart filtering to prevent duplicate runs
- Auto-review only for external contributors to reduce noise
- Implement sticky comments for PR updates

**Performance Improvements:**
- ~40% faster CI execution by eliminating job overhead
- Better cache hit rates with improved cache strategies
- Reduced compute costs through parallel execution
- Eliminated redundant dependency installations

**Documentation:**
- Add comprehensive workflow README with migration notes
- Document optimization results and usage patterns

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>

### 🚀 Features

* add BSP split ratio configuration option ([e397dab](https://github.com/d-matsui/rustile/commit/e397dabe66b9b1ff8268c700231077edcb932ca8))
* enable Claude auto-review for all PRs instead of external contributors only ([75ef5ee](https://github.com/d-matsui/rustile/commit/75ef5ee776230cedbc88ccf5cac41b2e0687ead7))
* implement BSP layout inspired by yabai window manager ([5c84646](https://github.com/d-matsui/rustile/commit/5c84646001978b9151ec5f11b0272c7929f5427c))
* optimize CI workflow for 40% faster execution ([0e91ea8](https://github.com/d-matsui/rustile/commit/0e91ea8e2ed135fc19671357eaad0fcfa25ae4bf))

### 🐛 Bug Fixes

* add comprehensive validation for all config parameters ([f63f87a](https://github.com/d-matsui/rustile/commit/f63f87a636571da51d1e9f4f9e495ec9b2c799d7))
* address final minor nitpick issues from PR review ([62e2cb7](https://github.com/d-matsui/rustile/commit/62e2cb7a0a3dd0c48bb760751ce2c4ca973b0a76))
* address minor nitpick issues from PR review ([fb208d8](https://github.com/d-matsui/rustile/commit/fb208d82d57077fe662329fe8d6835149728a4f0))
* implement configurable BSP split ratio as required by PR review ([6fd2133](https://github.com/d-matsui/rustile/commit/6fd21330121600f541179bdfa9be6fbfb65dcfe0))
* improve cargo-audit caching with direct file check ([72adb34](https://github.com/d-matsui/rustile/commit/72adb340ecc55eec6d876d6c063108da8301c2c3))
* remove unimplemented bsp_split_ratio from config documentation ([92741c7](https://github.com/d-matsui/rustile/commit/92741c7694798ecb88b237cba2506d72b58696e4))
* rename CI job to match required Test Suite status check ([5f73248](https://github.com/d-matsui/rustile/commit/5f73248bfa3599f0415c1ec69bc1a8d0e0056026))
* resolve CI clippy failure with uninlined format args ([4d6ae5e](https://github.com/d-matsui/rustile/commit/4d6ae5ed6fc6554850ae4c9af571130210226b63))
* resolve CI security audit caching issue ([bb30b32](https://github.com/d-matsui/rustile/commit/bb30b324ca4926570220a47f6061a9270527cd7f))

### 📖 Documentation

* add CODE_EXPLANATION.md to .gitignore and update with Japanese content ([edc346e](https://github.com/d-matsui/rustile/commit/edc346e55b20f3e1c59021b7d4e4cedbff0c9a48))
* remove contributor-specific language for personal project ([6b01054](https://github.com/d-matsui/rustile/commit/6b01054308a650464ee929784d2b81d07f6b07c5))
* remove unused develop branch references and clarify GitHub Flow ([8a85cef](https://github.com/d-matsui/rustile/commit/8a85cef17c8e5542164434364b2b6f856b20b581))
* update CLAUDE.md with correct CI-aligned clippy command ([b0bd1cd](https://github.com/d-matsui/rustile/commit/b0bd1cd34559d4f3c97921b154320bdae5685387))
* update CODE_EXPLANATION.md with latest features ([a59b1c2](https://github.com/d-matsui/rustile/commit/a59b1c2c40663ecd47f755c799c24f7ba7ef9cca))

### ♻️ Refactor

* comprehensive codebase cleanup and organization ([00c5fd1](https://github.com/d-matsui/rustile/commit/00c5fd199e51beb693d11145ab5696a3a56c59cd))
* consolidate documentation and remove redundant scripts ([251d208](https://github.com/d-matsui/rustile/commit/251d208cb6b39de1800c8a55fca88754d3a97c82))
* integrate layout switching into dev-tools.sh and remove standalone script ([63d98c3](https://github.com/d-matsui/rustile/commit/63d98c36919fa414054f71725b131b2b2f35445e))

### 🧪 Tests

* add comprehensive BSP layout edge case tests ([374ae1a](https://github.com/d-matsui/rustile/commit/374ae1ae64c8a8f378c850e597d6fd4cf33c5b2b))
* improve layout testing and switching infrastructure ([1e053ef](https://github.com/d-matsui/rustile/commit/1e053efdd64a75dc01abbbc7641e7849d23135ef))

## [0.2.0](https://github.com/d-matsui/rustile/compare/v0.1.0...v0.2.0) (2025-07-22)

### 🚀 Features

* implement automated semantic release system ([5b58963](https://github.com/d-matsui/rustile/commit/5b58963616770466a870ce4c4349be14873077c1))
* implement configurable gap system for window spacing ([fbe6ac3](https://github.com/d-matsui/rustile/commit/fbe6ac3b9d604c5f5e070ce3ac4149ea1406d5de))
* implement window focus management with visual indication ([5430fe3](https://github.com/d-matsui/rustile/commit/5430fe343c4a1b29e88edcca4ddf279294268a2f))

### 🐛 Bug Fixes

* address PR review feedback for gap system robustness ([c0c32ef](https://github.com/d-matsui/rustile/commit/c0c32eff32d3e3d21b8d8ae212eb0d9b2582f33b))
* improve gap system robustness and validation ([8968dd8](https://github.com/d-matsui/rustile/commit/8968dd8fc5c4c5e87fa7621e21f631a4d2ceebaa))
* resolve semantic-release timestamp parsing error ([e0dfc43](https://github.com/d-matsui/rustile/commit/e0dfc4323fbc04bf13ad851e39ca5124e7f47739))
* use latest semantic-release plugin versions without explicit version pins ([758cf94](https://github.com/d-matsui/rustile/commit/758cf94cd82e142d2ce2d43e979da3bbba98fff7))

### 📖 Documentation

* add comprehensive development rules and conventions to CLAUDE.md ([5159138](https://github.com/d-matsui/rustile/commit/51591380706d60251c7c8fff05621431bc3f898d))
* add configuration troubleshooting guide ([a67e539](https://github.com/d-matsui/rustile/commit/a67e539d4731c1d14ca67689ab10740c88ae209d))
* update README with gap system and configuration improvements ([7235afd](https://github.com/d-matsui/rustile/commit/7235afd6d9fd0ec6e77c44ecbe20bdf868fdf6df))

### 🔧 CI/CD

* fix automated version and changelog generation ([47acc73](https://github.com/d-matsui/rustile/commit/47acc73103aa2f966602e013c367786849b63360))
* fix semantic-release cargo set-version command failure ([f531be8](https://github.com/d-matsui/rustile/commit/f531be85fc72b37c310dc972ef340118f3d1f43a))
