# Performance / Ergonomics (Optional)

- Improve `ServerSettings::descriptors()` performance
  - Problem: `ServerSettings::descriptors()` currently constructs a new `Vec` on every call. This is fine for small descriptor sets, but can become inefficient if there are many descriptors and `get_by_key`/`set_by_key` are called frequently.
  - Proposed fixes:
    - Cache descriptors in a `HashMap` keyed by descriptor key to allow O(1) lookups.
    - Use a lazily-initialized static map (e.g., `once_cell::sync::Lazy` or `lazy_static!`) if descriptors are effectively static.
  - Considerations: memory overhead, invalidation strategy if descriptors change at runtime, and thread-safety. Add microbenchmarks to confirm any improvement.
  - Priority: Low/Medium

- Support `LocalizedText` labels in `SettingDescriptor`
  - Problem: If `SettingDescriptor` in the meta crate uses `LocalizedText` (a key + fallback) instead of `String` for labels, the current `server_port_descriptor()` implementation needs to be adapted.
  - Proposed fixes:
    - Update `server_port_descriptor()` to produce `LocalizedText` labels, or provide a conversion utility between `String` and `LocalizedText`.
  - Next step: if you provide the exact `LocalizedText` signature, I can prepare an updated snippet for `server_port_descriptor()`.
  - Priority: Low
