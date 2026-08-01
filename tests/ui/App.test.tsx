// Plantilla de test UI (Fase 0): smoke test de render con Testing Library.
// `invoke` de Tauri se mockea — en jsdom no hay backend real detrás.
import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue("mocked"),
}));

import App from "../../src/App";

describe("App", () => {
  it("renders the greet form", () => {
    render(<App />);
    expect(screen.getByPlaceholderText(/enter a name/i)).toBeInTheDocument();
  });
});
