use rand::random;
use std::io;
use std::str;

mod consts;
use fakenet::*; // Import every function from lib

// Aplicação: apenas escreve uma mensagem (string) e manda o conteúdo para a próxima camada
fn aplicacao_transmissora() {
    let mut message = String::new();
    println!("Digite uma mensagem!");
    io::stdin()
        .read_line(&mut message)
        .expect("Leitura do STDIN falhou");

    camada_de_aplicacao_transmissora(&message);
}

// Camada de aplicação: converte a mensagem para bytes (UTF-8) e manda para a camada seguinte
fn camada_de_aplicacao_transmissora(message: &str) {
    let bytes = message.as_bytes();

    println!("Convertendo {} em bytes: {:?}", message, bytes);

    camada_de_enlace_transmissora(&bytes);
}

// Enlace: Recebe a mensagem em bytes e converte em bits.
// Em seguida, aplica um dos métodos de controle de erros (paridade ímpar/par ou CRC)
fn camada_de_enlace_transmissora(bytes: &[u8]) {
    let bits = bytes_to_bits(bytes);

    println!("Convertendo {:?} em bits: {:?}", bytes, bits);

    let bits_extended = match consts::CONTROLE {
        0 => Ok(add_parity_check(&bits, consts::CHUNKSIZE, consts::CONTROLE)), // Paridade par
        1 => Ok(add_parity_check(&bits, consts::CHUNKSIZE, consts::CONTROLE)), // Paridade ímpar
        2 => Ok(append_crc_hash(&bits)),                                       // CRC
        _ => Err("Apenas valores de 0 a 2 são válidos para controle de erros"), // Outros métodos não podem ser utilizados
    };

    let bits_extended = bits_extended.unwrap();
    println!(
        "Verificação de erro: stream {:?} se torna {:?}",
        bits, bits_extended
    );
    meio_de_comunicacao(&bits_extended);
}

// Propaga os bits tais como recebidos para o receptor
// Dependendo da constante ERRO, alguns bits 1 podem não ser registrados
fn meio_de_comunicacao(bits: &[u8]) {
    let bits_transmitted: Vec<u8> = bits
        .iter()
        .map(|bit| bit * ((consts::ERRO < random()) as u8))
        .collect();
    println!("Erros no meio de transmissão: {}%", consts::ERRO * 100.0);
    println!(
        "Stream original: {:?}\nStream com erros: {:?}",
        bits, bits_transmitted
    );
    camada_de_enlace_receptora(&bits_transmitted);
}

// Recebe os bits do meio de transmissão e verifica usando o método de controle de erros escolhido
// Gera erro caso não seja possível recuperar a informação
fn camada_de_enlace_receptora(bits: &[u8]) {
    let original_message = match consts::CONTROLE {
        0 => check_parity(bits, consts::CHUNKSIZE, consts::CONTROLE),
        1 => check_parity(bits, consts::CHUNKSIZE, consts::CONTROLE),
        2 => recover_from_crc_hash(bits),
        _ => Err(String::from(
            "Apenas valores de 0 a 2 são válidos para controle de erros",
        )),
    };

    let original_message = original_message.unwrap();
    println!(
        "Recebido pela camada de enlace do receptor: {:?}\nInterpretado: {:?}",
        bits, original_message
    );
    let bytes = bits_to_bytes(&original_message);
    camada_de_aplicacao_receptora(&bytes);
}

fn camada_de_aplicacao_receptora(bytes: &[u8]) {
    let string = String::from_utf8(bytes.to_vec()).unwrap();
    println!("Bytes recebidos pela aplicação: {:?}", bytes);
    println!("Mensagem recebida: {}", string);
}

fn main() {
    aplicacao_transmissora();
}
