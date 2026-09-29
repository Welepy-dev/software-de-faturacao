//! Conversão de valores monetários para texto por extenso em português —
//! requisito do artigo 10.º n.º1 al. d) do Decreto 71/25 (docs/AGT-SAFT.md
//! §0): o total do documento tem de aparecer, além de em algarismos,
//! escrito por extenso.
//!
//! Implementação pragmática: cobre valores até 999.999.999 unidades
//! (milhões) — suficiente para qualquer factura realista. Acima disso
//! devolve uma indicação em algarismos em vez de arriscar uma
//! nomenclatura errada (evitar a confusão português "bilião" = 10^12 vs.
//! inglês "billion" = 10^9).

const UNIDADES: [&str; 20] = [
    "zero", "um", "dois", "três", "quatro", "cinco", "seis", "sete", "oito", "nove", "dez",
    "onze", "doze", "treze", "catorze", "quinze", "dezasseis", "dezassete", "dezoito",
    "dezanove",
];

const DEZENAS: [&str; 10] = [
    "", "", "vinte", "trinta", "quarenta", "cinquenta", "sessenta", "setenta", "oitenta",
    "noventa",
];

const CENTENAS: [&str; 10] = [
    "", "cento", "duzentos", "trezentos", "quatrocentos", "quinhentos", "seiscentos",
    "setecentos", "oitocentos", "novecentos",
];

fn extenso_0_99(n: u32) -> String {
    if n < 20 {
        UNIDADES[n as usize].to_string()
    } else {
        let d = (n / 10) as usize;
        let u = n % 10;
        if u == 0 {
            DEZENAS[d].to_string()
        } else {
            format!("{} e {}", DEZENAS[d], UNIDADES[u as usize])
        }
    }
}

fn extenso_0_999(n: u32) -> String {
    if n == 100 {
        return "cem".to_string();
    }
    let c = (n / 100) as usize;
    let r = n % 100;
    match (c, r) {
        (0, _) => extenso_0_99(r),
        (_, 0) => CENTENAS[c].to_string(),
        (_, _) => format!("{} e {}", CENTENAS[c], extenso_0_99(r)),
    }
}

/// Converte um inteiro não negativo (até 999.999.999) para português.
fn extenso_inteiro(n: u64) -> String {
    if n == 0 {
        return "zero".to_string();
    }
    if n >= 1_000_000_000 {
        return format!("{n}"); // fallback — fora do intervalo suportado
    }

    let milhoes = (n / 1_000_000) as u32;
    let milhares = ((n / 1000) % 1000) as u32;
    let unidades = (n % 1000) as u32;

    let mut grupos: Vec<String> = Vec::new();
    if milhoes > 0 {
        if milhoes == 1 {
            grupos.push("um milhão".to_string());
        } else {
            grupos.push(format!("{} milhões", extenso_0_999(milhoes)));
        }
    }
    if milhares > 0 {
        if milhares == 1 {
            grupos.push("mil".to_string());
        } else {
            grupos.push(format!("{} mil", extenso_0_999(milhares)));
        }
    }
    if unidades > 0 {
        grupos.push(extenso_0_999(unidades));
    }

    match grupos.len() {
        1 => grupos.remove(0),
        _ => {
            // "e" antes do último grupo quando este é < 100 (regra comum
            // do português para números por extenso); "," nos restantes.
            let ultimo = grupos.pop().unwrap();
            let resto = grupos.join(", ");
            if unidades > 0 && unidades < 100 {
                format!("{resto} e {ultimo}")
            } else {
                format!("{resto}, {ultimo}")
            }
        }
    }
}

fn concordar(n: u64, singular: &str, plural: &str) -> String {
    if n == 1 {
        singular.to_string()
    } else {
        plural.to_string()
    }
}

/// Valor por extenso, ex.: `valor_por_extenso(123456, "kwanza", "kwanzas",
/// "cêntimo", "cêntimos")` → "mil, duzentos e trinta e quatro kwanzas e
/// cinquenta e seis cêntimos" (para 1234.56).
pub fn valor_por_extenso(
    total_centimos: i64,
    moeda_singular: &str,
    moeda_plural: &str,
    centimo_singular: &str,
    centimo_plural: &str,
) -> String {
    let total_centimos = total_centimos.unsigned_abs();
    let unidades = total_centimos / 100;
    let centimos = total_centimos % 100;

    let parte_unidades = format!(
        "{} {}",
        extenso_inteiro(unidades),
        concordar(unidades, moeda_singular, moeda_plural)
    );

    if centimos == 0 {
        parte_unidades
    } else {
        format!(
            "{} e {} {}",
            parte_unidades,
            extenso_inteiro(centimos),
            concordar(centimos, centimo_singular, centimo_plural)
        )
    }
}
