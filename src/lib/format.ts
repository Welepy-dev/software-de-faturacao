// Formatação de valores como nos mockups de "Faturacao v3":
// "1 234,50 Kz", com espaço inseparável como separador de milhares.

export function fmt(centimos: number): string {
  const neg = centimos < 0;
  const c = Math.abs(Math.round(centimos));
  const inteiro = String(Math.floor(c / 100)).replace(/\B(?=(\d{3})+(?!\d))/g, " ");
  return (neg ? "−" : "") + inteiro + "," + String(c % 100).padStart(2, "0") + " Kz";
}

export function fmtS(centimos: number): string {
  return (centimos > 0 ? "+" : "") + fmt(centimos);
}

/** Converte texto introduzido pelo operador ("20 000", "1.500,50") em cêntimos. */
export function parseKz(texto: string): number {
  const n = parseFloat(
    String(texto).replace(/[\s ]/g, "").replace(/\./g, "").replace(",", "."),
  );
  return isNaN(n) ? 0 : Math.round(n * 100);
}

export function qtyStr(q: number): string {
  return String(+(+q).toFixed(3)).replace(".", ",");
}

export function horaMinuto(d: Date | string): string {
  const x = typeof d === "string" ? new Date(d) : d;
  return String(x.getHours()).padStart(2, "0") + ":" + String(x.getMinutes()).padStart(2, "0");
}

export function dataCurta(d: Date): string {
  return d.toLocaleDateString("pt-AO", { day: "2-digit", month: "2-digit", year: "numeric" });
}

export function iniciais(nome: string): string {
  const partes = nome.trim().split(/\s+/);
  return ((partes[0]?.[0] ?? "") + (partes.length > 1 ? partes[partes.length - 1][0] : "")).toUpperCase();
}

export function diffColor(d: number): string {
  return d === 0 ? "#15803D" : d < 0 ? "#B91C1C" : "#B45309";
}

export function norm(s: string): string {
  return s.toLowerCase().normalize("NFD").replace(/[̀-ͯ]/g, "");
}

/** Mensagem legível de um erro vindo de `invoke` (o backend devolve strings). */
export function erroTexto(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : String(e);
}
