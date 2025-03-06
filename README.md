## VSEBINSKI DEL (TRGOVANJE)

### Projektna zasnova in cilji

Razvoj arhitekture za algoritemsko trgovanje na platformi Binance s terminskimi pogodbami z naslednjimi ključnimi komponentami:

1. **Sistemska arhitektura**

   - Real-Time monitoring tržnih podatkov
   - Analiza Trenda (določanje bull/bear tržnih stanj)
   - Risk management sistem:
     - Dinamično upravljanje pozicij (glede na stanje marketa)
     - Stop-loss mehanizmi
     - Take-profit mehanizmi
     - Leverage optimizacija

2. **Strategija izvrševanja**
   - Na podlagi implementacije algoritma, na podlagi katerega bojo izvrseni orderji:
     - Avtomatsko postavljanje limit/market orderji
     - Balansiranje izpostavljenosti
     - Likvidnostno upravljanje
     - Logiranje orderjev in transakcij
     - Obdelava napak in opozoril
     - Nadzor nad stanjem pozicij

---

### Motivacija

Ključna motivacija projekta je študij sodobnih pristopov k razvoju finančnih trading sistemov v Rust ekosistemu.

---

1. **OSNOVNA INFRASTRUKTURA**

   - Algoritem za trgovanje

     - Na zacetku bo to simple SMA cross strategije (drseči povprečji)
     - Ko vspostavimo vso ostalo arhitekturo, se bomo posvetili strategiji, ki bo bolj kompleksna (ce bo cas)

   - Docker container z bazo za shranjevanje:
     - Log orderjev
     - Log napak in opozoril
     - Time-Series podatkov (mogoce ja mogoce ne, odvisno od strategije)

2. **POVEZAVA NA BINANCE**

   - WebSocket za real-time cenovne podatke:
     - Vzdrževanje stabilne povezave
     - Obdelava prekinitev in ponovne vzpostavitve
     - Postavljanje limit orderjev

3. **ANALIZA ČASOVNIH VRST**
   - Implementacija finančnih indikatorjev
