# BMW Toolbox — Principios del proyecto

No es un documento técnico. Es la brújula del proyecto: lo que se consulta cuando aparece un atajo tentador o una decisión ambigua. Tiene más autoridad que cualquier preferencia puntual de velocidad — si algo en `PROJECT_PLAN.md` entra en conflicto con esto, este documento gana.

---

## Principios de diseño

1. **Simplicidad antes que complejidad.** La solución más simple que cumple el requisito real gana, siempre. Complejidad se añade solo cuando un requisito concreto la exige, nunca por anticipación especulativa.
2. **Calidad antes que rapidez.** "Ya funciona" no es el criterio de aceptación. El criterio es "funciona bien": maneja errores, está probado, es legible por otra persona sin contexto previo.
3. **Seguridad antes que funcionalidad.** Ninguna función nueva se implementa si compromete la garantía central del proyecto: V1 no escribe nada en ninguna centralita.
4. **Documentación antes que memoria.** Ninguna decisión importante vive solo en la cabeza de quien la tomó. Si no está escrita, no ha pasado.
5. **Modularidad antes que acoplamiento.** Un módulo se puede sustituir o eliminar sin que el resto del sistema note más que la ausencia de esa funcionalidad.
6. **Cero deuda técnica intencionada.** No se escribe código "para arreglar después" sin que quede registrado explícitamente como tal y con plan de arreglo. Un atajo consciente y documentado es aceptable en casos justificados; un atajo silencioso, no.
7. **Todo debe poder probarse sin el vehículo.** Si una pieza de lógica solo se puede validar teniendo el coche delante, algo está mal diseñado — falta una capa de abstracción o un mock.
8. **Ninguna decisión irreversible sin aprobación explícita.** Cambios de arquitectura, de stack, o cualquier cosa cara de deshacer se proponen y esperan luz verde antes de aplicarse.
9. **Cada módulo debe poder sustituirse sin romper el sistema.** Si sustituir un módulo obliga a tocar código de otro módulo, la frontera entre ambos está mal trazada.
10. **El código debe ser entendible por alguien nuevo dentro de cinco años.** No solo por quien lo escribió la semana pasada.
11. **Dependencias antes que comodidad.** Cada nueva dependencia debe justificar claramente qué problema resuelve. Antes de añadir una librería se evalúa si el problema puede resolverse razonablemente con las herramientas ya existentes del proyecto. Se evitan dependencias poco mantenidas, excesivamente pesadas, o de un único propósito trivial cuando aportan más complejidad que valor.
12. **Compatibilidad antes que ruptura.** Siempre que sea posible, las nuevas funcionalidades mantienen compatibilidad con APIs, configuraciones y datos existentes (en particular, con el esquema de la base de datos y las migraciones ya aplicadas). Cuando una ruptura sea inevitable, se documenta con un ADR y se acompaña de un plan de migración explícito.

---

## Reglas de implementación (no negociables)

- **Ninguna capa se salta a otra.** La UI nunca habla directamente con SQLite ni con `Transport`. Un módulo nunca accede al estado interno de otro módulo. Todo pasa por la interfaz pública correspondiente (comando Tauri, API de módulo, trait). Un PR que salte una capa se rechaza, sin excepción de "es solo para probar rápido".
- **Prohibido el condicional por identidad de marca/módulo en lógica de negocio.** Nada de `if manufacturer == "BMW" { ... }` disperso por el código. La variación de comportamiento se resuelve con polimorfismo: traits (`Transport`, `Protocol`, `Module`) e implementaciones concretas (`Bmw`, `EnetTransport`, `UdsProtocol`). Si aparece la tentación de un `if`/`match` sobre el nombre de una marca o módulo fuera de un punto de registro explícito (`ModuleRegistry`, factoría de dominio), es una señal de que falta una abstracción — se rediseña esa pieza antes de continuar.
- **Toda decisión arquitectónica relevante genera un ADR**, no solo un cambio de código. "Relevante" significa: elección de librería, patrón estructural nuevo, cambio de una decisión ya tomada, o cualquier cosa que alguien preguntaría "¿por qué se hizo así?" dentro de un año.
- **La prueba del "dentro de cinco años"**: al implementar algo con una decisión de diseño no trivial, la pregunta explícita es *¿seguiría haciendo esto igual dentro de cinco años, con más módulos, más marcas y más usuarios?* Si la respuesta es no, se rediseña antes de seguir, no después.

---

## Regla de refactorización y reconsideración de decisiones (dirigida al desarrollo asistido por Claude)

Si en cualquier momento del desarrollo se detecta que una decisión tomada anteriormente (en este documento, en `PROJECT_PLAN.md`, en un ADR, o en código ya escrito) ya no es la mejor opción — por nueva información, cambio de requisitos, o una limitación técnica descubierta durante la implementación — la regla es:

1. **Detener el desarrollo de esa parte concreta.** No seguir avanzando sobre una base que ya se considera inferior.
2. **Explicar el problema con claridad:** qué se asumía antes, qué ha cambiado, por qué la decisión original ya no sirve.
3. **Proponer alternativas concretas**, con sus trade-offs — igual que en el proceso de diseño inicial.
4. **Esperar aprobación explícita** antes de continuar la implementación.

Nunca se continúa implementando una solución que se considera inferior solo por fidelidad al plan inicial. El plan es un punto de partida informado, no una promesa que deba cumplirse a pesar de la evidencia. Esta regla tiene prioridad sobre "seguir el roadmap tal cual está escrito".

---

## Norma final: calidad de producto antes que velocidad de roadmap

Durante todo el desarrollo, se prioriza siempre construir un producto excelente antes que completar el roadmap rápidamente:

- Si aparece una solución mejor que la planificada, se propone el cambio (ver regla de reconsideración arriba).
- Si se detecta un problema de diseño, se detiene el trabajo y se comunica — no se avanza "para no perder el ritmo".
- Si una funcionalidad no alcanza el nivel de calidad esperado (sección "Reglas de implementación"), no se da por terminada solo por cumplir la fecha o el orden del roadmap.
- La calidad del producto tiene prioridad sobre la velocidad de desarrollo. El roadmap describe el orden del trabajo, no una promesa de plazos a cualquier coste.

---

## Cómo se usa este documento

- Se lee antes de tomar cualquier decisión de diseño no trivial durante el desarrollo.
- Se cita explícitamente en un ADR o en una propuesta de cambio cuando un principio concreto es la razón de una decisión ("Principio 6: cero deuda técnica intencionada" es más útil que "por limpieza").
- No se modifica a la ligera. Si un principio deja de tener sentido, se discute y se cambia con la misma seriedad que una decisión arquitectónica — con la razón documentada, no solo borrado.
