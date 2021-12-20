// Escolha o método descomentando a linha adequada e descomentando as outras
// pub const CONTROLE: u8 = 0;  // Paridade (par)
pub const CONTROLE: u8 = 1; // Paridade (ímpar)
                            // pub const CONTROLE: u8 = 2;  // CRC

pub const CHUNKSIZE: usize = 5; // Quantidade de bits por linha na verificação por paridade

pub const ERRO: f64 = 0.0; // Razão de bits com erro
