# Frontend architecture

The Vue frontend is organized around a small number of dependency layers:

```text
views / components
       |
       v
stores / composables
       |
       v
api (Tauri commands and external translation service)
       |
       v
types
```

Dependencies should flow downward. Views may coordinate stores and
composables, while reusable interaction state belongs in a composable rather
than in `App.vue` or an individual view.

## Directories

- `views/`: route-level screens and overlay-capable detail/reader screens.
- `components/`: reusable presentation components.
- `stores/`: shared application state and domain workflows.
- `composables/`: reusable browser or Vue lifecycle behavior.
- `api/`: the public frontend gateway to Tauri commands and external services.
- `types/`: serializable domain contracts shared across the frontend.

## API modules

Consumers should import from `@/api`. The barrel keeps call sites stable while
the implementations are grouped by responsibility:

- `settings.ts`: settings, authentication, and Cloudflare state.
- `gallery.ts`: remote gallery/search API calls.
- `library.ts`: local favorites, tags, history, progress, and downloaded items.
- `downloads.ts`: download commands and progress events.
- `system.ts`: file/export/application operations and release checks.
- `translation.ts`: the OpenAI-compatible translation service client.

Each wrapper maps directly to a backend command unless the module explicitly
documents an external HTTP service. Avoid invoking Tauri commands directly in
views; adding the wrapper here keeps transport details out of UI code.

## Application shell

`App.vue` composes global stores and renders the application shell. Generic
interaction state is kept in focused composables:

- `useResponsiveSidebar`: compact breakpoint and drawer state.
- `useEdgeSwipe`: left-edge gesture recognition.
- `useSwipeDismiss`: touch dismissal for overlay panels.
- `useDraggablePosition`: pointer dragging and persisted viewport position.

When adding a new shell interaction, prefer a composable if it owns a distinct
state machine or lifecycle. Keep feature data and business rules in a store.
