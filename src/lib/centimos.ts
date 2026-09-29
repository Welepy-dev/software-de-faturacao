// Valores monetários viajam sempre em cêntimos (inteiros) entre o
// frontend e o backend Rust — ver convenção em src-tauri/migrations/0001_init.sql.

export function centimosParaMoeda(centimos: number, moeda = "AOA"): string {
  return new Intl.NumberFormat("pt-AO", { style: "currency", currency: moeda }).format(
    centimos / 100,
  );
}

export function moedaParaCentimos(valor: number): number {
  return Math.round(valor * 100);
}
