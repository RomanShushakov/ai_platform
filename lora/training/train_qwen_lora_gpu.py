import json
import os
from pathlib import Path

import torch
from datasets import Dataset
from peft import LoraConfig, get_peft_model
from transformers import (
    AutoModelForCausalLM,
    AutoTokenizer,
    Trainer,
    TrainingArguments,
)

BASE_MODEL = os.environ.get(
    "BASE_MODEL",
    "/home/roman/nfs/models/huggingface/Qwen2.5-0.5B-Instruct",
)
DATASET_PATH = os.environ.get(
    "DATASET_PATH",
    "/home/roman/nfs/lora/datasets/lab_style.jsonl",
)
OUTPUT_DIR = os.environ.get(
    "OUTPUT_DIR",
    "/home/roman/nfs/lora/adapters/lab_style_qwen2_5_0_5b_gpu",
)
MAX_LENGTH = int(os.environ.get("MAX_LENGTH", "256"))


def load_jsonl(path: str):
    rows = []

    with open(path, "r", encoding="utf-8") as f:
        for line in f:
            line = line.strip()

            if not line:
                continue

            rows.append(json.loads(line))

    return rows


def main():
    print(f"BASE_MODEL={BASE_MODEL}")
    print(f"DATASET_PATH={DATASET_PATH}")
    print(f"OUTPUT_DIR={OUTPUT_DIR}")
    print(f"CUDA={torch.cuda.is_available()}")
    print(f"CUDA_DEVICE_COUNT={torch.cuda.device_count()}")

    if not torch.cuda.is_available():
        raise RuntimeError("CUDA is not available inside the training container")

    Path(OUTPUT_DIR).mkdir(parents=True, exist_ok=True)

    tokenizer = AutoTokenizer.from_pretrained(BASE_MODEL, trust_remote_code=True)

    if tokenizer.pad_token is None:
        tokenizer.pad_token = tokenizer.eos_token

    rows = load_jsonl(DATASET_PATH)
    dataset = Dataset.from_list(rows)

    def tokenize(example):
        text = tokenizer.apply_chat_template(
            example["messages"],
            tokenize=False,
            add_generation_prompt=False,
        )

        encoded = tokenizer(
            text,
            truncation=True,
            max_length=MAX_LENGTH,
            padding="max_length",
        )

        encoded["labels"] = encoded["input_ids"].copy()
        return encoded

    tokenized = dataset.map(tokenize, remove_columns=dataset.column_names)

    model = AutoModelForCausalLM.from_pretrained(
        BASE_MODEL,
        trust_remote_code=True,
        torch_dtype=torch.float16,
        device_map={"": 0},
    )

    model.config.use_cache = False

    lora_config = LoraConfig(
        r=8,
        lora_alpha=16,
        target_modules=[
            "q_proj",
            "k_proj",
            "v_proj",
            "o_proj",
            "gate_proj",
            "up_proj",
            "down_proj",
        ],
        lora_dropout=0.05,
        bias="none",
        task_type="CAUSAL_LM",
    )

    model = get_peft_model(model, lora_config)
    model.print_trainable_parameters()

    args = TrainingArguments(
        output_dir=OUTPUT_DIR,
        num_train_epochs=float(os.environ.get("NUM_TRAIN_EPOCHS", "1")),
        max_steps=int(os.environ.get("MAX_STEPS", "1")),
        per_device_train_batch_size=int(os.environ.get("BATCH_SIZE", "1")),
        gradient_accumulation_steps=int(os.environ.get("GRAD_ACCUM_STEPS", "1")),
        learning_rate=float(os.environ.get("LEARNING_RATE", "2e-4")),
        logging_steps=1,
        save_steps=1,
        save_total_limit=1,
        report_to=[],
        fp16=True,
    )

    trainer = Trainer(
        model=model,
        args=args,
        train_dataset=tokenized,
    )

    trainer.train()

    model.save_pretrained(OUTPUT_DIR)
    tokenizer.save_pretrained(OUTPUT_DIR)

    print(f"Saved GPU LoRA adapter to {OUTPUT_DIR}")


if __name__ == "__main__":
    main()
