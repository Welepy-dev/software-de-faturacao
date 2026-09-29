import { invoke } from "@tauri-apps/api/core";
import type { Estabelecimento } from "../../lib/types";

export function listarEstabelecimentos(): Promise<Estabelecimento[]> {
  return invoke("listar_estabelecimentos");
}
