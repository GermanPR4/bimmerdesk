# BMW Toolbox — Sistema de diseño

Referencias: BMW, Tesla, VS Code, Discord, Steam, Adobe, JetBrains. Oscuro por defecto, limpio, denso en información pero sin sobrecarga visual, animaciones de transición de estado (nunca decorativas). Implementado como tokens CSS en `src/styles/tokens.css` — este documento explica el porqué de cada valor; el CSS es la fuente ejecutable.

## Color

Escala de grises fría de base (azulados, no negros puros — un negro puro sobre pantallas OLED/LCD reales se siente "roto", los productos de referencia usan grises muy oscuros) + un acento único.

| Token | Valor | Uso |
|---|---|---|
| `--color-bg-canvas` | `#0b0d10` | Fondo de la ventana, detrás de todo. |
| `--color-bg-surface` | `#14171b` | Paneles, sidebar, cards. |
| `--color-bg-surface-raised` | `#1c2026` | Elementos elevados sobre una surface (modales, dropdowns). |
| `--color-border` | `#2a2f36` | Bordes sutiles entre superficies. |
| `--color-border-strong` | `#3a4048` | Bordes con más énfasis (inputs enfocados, divisores importantes). |
| `--color-text-primary` | `#e8eaed` | Texto principal. Nunca blanco puro — cansa menos en sesiones largas. |
| `--color-text-secondary` | `#9aa1ab` | Texto secundario, labels, metadatos. |
| `--color-text-disabled` | `#5c636d` | Texto/iconos deshabilitados. |
| `--color-accent` | `#3b82f6` | Acento único: acciones primarias, estados activos, foco. Azul — no el azul BMW corporativo (evita implicar afiliación oficial con la marca), pero coherente con la paleta "tech" de las referencias. |
| `--color-accent-hover` | `#5b93f7` | Hover del acento. |
| `--color-success` | `#2fb379` | Confirmaciones, "OK", conexión establecida. |
| `--color-warning` | `#d99a3d` | Avisos — ej. DTC de severidad media. |
| `--color-danger` | `#e5484d` | Errores, DTC crítico, desconexión inesperada. Reservado — nunca se usa para nada que no sea un problema real (si todo es rojo, nada destaca). |

**Regla de uso:** el acento (`--color-accent`) se usa con moderación — botones primarios, elementos activos, foco de teclado. Si una pantalla tiene más de 2-3 elementos en acento a la vez, probablemente está sobrecargada.

## Tipografía

- **Familia:** fuente del sistema (`-apple-system, "Segoe UI", Roboto, sans-serif`) para UI; `"Cascadia Code", "JetBrains Mono", monospace` para datos técnicos (VIN, DTCs, valores en vivo, hex). Sin fuentes descargadas de terceros en V1 — menos peso, menos superficie de fallo, consistente con Principio 11 (dependencias antes que comodidad).
- **Escala:** `--font-size-xs` (12px, metadatos) · `--font-size-sm` (13px, texto de UI por defecto) · `--font-size-md` (15px, contenido destacado) · `--font-size-lg` (19px, títulos de sección) · `--font-size-xl` (24px, títulos de pantalla).
- **Peso:** 400 por defecto, 600 para énfasis/títulos. Nunca 700+ — se siente pesado en una UI oscura densa en datos.

## Espaciado

Escala de 4px (`--space-1` = 4px ... `--space-8` = 32px), consistente con el grid de 4/8px que usan la mayoría de estas referencias. Evita valores "sueltos" (ej. 13px de padding) que no encajan en ningún ritmo visual.

## Radios y sombras

- **Radios:** `--radius-sm` (4px, inputs/botones pequeños) · `--radius-md` (8px, cards/paneles) · `--radius-lg` (12px, modales). Nada completamente cuadrado (se siente anticuado) ni muy redondeado (se siente "consumer app", no herramienta profesional).
- **Sombras:** mínimas y solo para elevación real (modales, dropdowns sobre contenido) — `--shadow-elevated: 0 8px 24px rgba(0,0,0,0.4)`. Nunca sombra decorativa en elementos planos de layout.

## Animación

- **Duración:** `--motion-fast` (120ms, hover/foco) · `--motion-base` (200ms, transiciones de estado — abrir panel, cambiar de pestaña) · `--motion-slow` (320ms, transiciones de pantalla completa).
- **Easing:** `cubic-bezier(0.4, 0, 0.2, 1)` (estándar "ease-out" de Material/mayoría de sistemas modernos) para todo — consistencia antes que variedad.
- **Regla:** anima estado (aparecer/desaparecer, expandir/colapsar, cambio de valor numérico en Live Data), nunca decoración (nada de animaciones de entrada en cada card al cargar una pantalla — eso ralentiza percibir la información, que es el objetivo real de la app).

## Iconografía

Set de iconos de trazo simple (estilo outline, 1.5-2px de grosor), consistente con VS Code/JetBrains — nunca iconos rellenos ni ilustrativos/coloridos (se sentiría "app de consumo", no herramienta de diagnóstico). Se elige la librería concreta (ej. Lucide) al construir el primer componente que la necesite, no antes — no bloquea el resto del sistema de diseño.

## Componentes primitivos (Fase 0 — solo tokens; componentes reales cuando el primer módulo los necesite)

`src/components/ui/` construye sobre estos tokens: `Button`, `Card`, `Input`, `Badge` (para severidad de DTC), `StatTile` (para el Dashboard). No se construyen en Fase 0 sin una pantalla real que los use — evita componentes especulativos sin caso de uso probado (PROJECT_PRINCIPLES.md, Principio 1: simplicidad antes que complejidad).
