use anyhow::Error as E;
use candle_core::{Device, IndexOp, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::bert::{BertModel, Config, DTYPE};
use tokenizers::{EncodeInput, InputSequence, Tokenizer};

pub struct Embedder {
    model: BertModel,
    tokenizer: Tokenizer,
    device: Device,
}

impl Embedder {
    pub fn new() -> Result<Self, E> {
        let device = Device::Cpu;
        let weights_filename = "resources/model.safetensors";
        let tokenizer_filename = "resources/tokenizer.json";
        let config = include_str!("../resources/config.json");

        let var_builder =
            unsafe { VarBuilder::from_mmaped_safetensors(&[weights_filename], DTYPE, &device)? };

        let config: Config = serde_json::from_str(&config)?;
        let mut tokenizer = Tokenizer::from_file(tokenizer_filename).map_err(E::msg)?;

        tokenizer
            .with_padding(Some(Default::default()))
            .with_truncation(None)
            .map_err(E::msg)?;

        let model = BertModel::load(var_builder, &config)?;

        Ok(Self {
            model,
            tokenizer,
            device,
        })
    }

    fn feed_forward<'s, T>(&'s self, prompt: T) -> Result<Tensor, E>
    where
        T: Into<EncodeInput<'s>>,
    {
        let tokens = self
            .tokenizer
            .encode(prompt, true)
            .map_err(E::msg)?
            .get_ids()
            .to_vec();

        let token_ids = Tensor::new(&tokens[..], &self.device)?.unsqueeze(0)?;
        let token_type_ids = token_ids.zeros_like()?;
        let tensor = self.model.forward(&token_ids, &token_type_ids, None)?;

        Ok(tensor)
    }

    /// Creates an embedding of the provided text.
    /// Currently uses the `all-MiniLM-L6-v2` Bert model.
    pub fn embed<'s, T>(&'s self, message: T) -> Result<Vec<f32>, E>
    where
        T: Into<InputSequence<'s>> + std::fmt::Debug,
    {
        println!("embed: {message:?}");
        let tensor = self.feed_forward(message)?;
        let v = tensor_to_vec(&tensor)?;
        Ok(v)
    }

    pub fn bulk_embed<'s, T>(&'s self, messages: &[T]) -> Result<Vec<Vec<f32>>, E>
    where
        T: AsRef<str>,
    {
        use tokenizers::Encoding;

        // 1. Tokenize all inputs
        let encodings: Vec<Encoding> = self
            .tokenizer
            .encode_batch(
                messages.iter().map(|s| s.as_ref()).collect::<Vec<_>>(),
                true, // add_special_tokens
            )
            .map_err(E::msg)?;

        // 2. Convert to token ID tensors
        let token_ids: Vec<Vec<u32>> = encodings.iter().map(|e| e.get_ids().to_vec()).collect();

        let token_tensor = Tensor::new(token_ids, &self.device)?;
        let token_type_ids = token_tensor.zeros_like()?;

        // 4. Forward pass
        let output = self.model.forward(&token_tensor, &token_type_ids, None)?;

        // 5. Reduce each sequence via mean pooling across tokens
        let output = output.squeeze(0)?; // (batch, seq_len, hidden)
        let mut embeddings = Vec::with_capacity(messages.len());

        for (i, encoding) in encodings.iter().enumerate() {
            let len = encoding
                .get_attention_mask()
                .iter()
                .filter(|&&m| m == 1)
                .count();
            let seq_tensor = output.i(i)?;
            let valid = seq_tensor.i(..len)?;
            let mean = valid.mean(0)?;
            embeddings.push(mean.to_vec1::<f32>()?);
        }

        Ok(embeddings)
    }
}

fn tensor_to_vec(tensor: &Tensor) -> anyhow::Result<Vec<f32>> {
    let tensor = tensor.squeeze(0)?;
    let mean = tensor.mean(0)?;
    let vec = mean.to_vec1::<f32>()?;
    Ok(vec)
}

#[cfg(test)]
mod test {
    use super::Embedder;
    use anyhow::Error as E;

    #[test]
    fn embed_similar_sentences() -> Result<(), E> {
        let embedder = Embedder::new()?;
        let rose = embedder.embed("five rose in a home")?;
        let tulip = embedder.embed("five roses in a house")?;
        let similarity = cosine_similarity(rose, tulip);
        println!("{similarity}");
        assert!(similarity > 0.8);
        Ok(())
    }

    #[test]
    fn embed_different_sentenct() -> Result<(), E> {
        let embedder = Embedder::new()?;
        let rose = embedder.embed("cat")?;
        let tulip = embedder.embed("tulip")?;
        let similarity = cosine_similarity(rose, tulip);
        println!("{similarity}");
        assert!(similarity < 0.5);
        Ok(())
    }

    fn cosine_similarity(vec1: Vec<f32>, vec2: Vec<f32>) -> f32 {
        let dot_product = vec1
            .iter()
            .zip(vec2.iter())
            .map(|(a, b)| a * b)
            .sum::<f32>();

        let norm1 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

        dot_product / (norm1 * norm2)
    }
}
