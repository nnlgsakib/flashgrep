## REMOVED Requirements

### Requirement: Embedded model bootstrap and cache lifecycle
**Reason**: Neural model download and cache management are being removed with AI-based navigation support.
**Migration**: Use lexical indexing/search workflows only; no model bootstrap or `.flashgrep/model-cache` setup is required.

### Requirement: Vector-backed semantic retrieval
**Reason**: Semantic vector retrieval is out of scope after removing neural search.
**Migration**: Use lexical query workflows and existing indexed text retrieval.

### Requirement: Deterministic neural output contract
**Reason**: Neural result contracts are no longer relevant once neural retrieval is removed.
**Migration**: Consume lexical query result contracts from `search-engine` and `cli-search-commands` specifications.
