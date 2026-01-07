(Optional) Performance / Ergonomie
- `ServerSettings::descriptors()` baut aktuell bei jedem Aufruf einen Vec neu auf. Für kleine Sets ist das OK; falls du viele Descriptoren hast und oft `get_by_key`/`set_by_key` aufrufst, könntest du ein cached HashMap oder eine lazy-static Map in Erwägung ziehen.
- Wenn `SettingDescriptor` in deinem meta-Crate Labels als `LocalizedText` (key + fallback) statt `String` verwendet, passe `server_port_descriptor()` entsprechend an (ich kann dir direkt das Snippet anpassen, wenn du mir die Exakte Signatur sagst).
