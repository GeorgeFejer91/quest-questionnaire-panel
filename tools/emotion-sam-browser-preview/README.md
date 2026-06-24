# Emotion SAM Browser Preview

Static browser preview for a counterbalanced four-condition emotion-induction
workflow followed by a repeated emotion assessment block.

Run locally:

```powershell
powershell -ExecutionPolicy Bypass -File .\tools\emotion-sam-browser-preview\Start-EmotionSamBrowserPreview.ps1
```

The preview is constrained to the workflow panel frame, `1080dp x 720dp`.

## Native Transfer Contract

The browser preview is not the product runtime. The native Android/Compose
panel and its `quest.questionnaire.v1` request/result handling remain
authoritative for launch, focus, result URI writing, completion callbacks, and
headset validation.

Counterbalancing is represented as request/caller-owned state:

- `counterbalance.order_id`
- `counterbalance.condition_ids`
- `condition.active_index`

The participant assessment block records:

- `sam.valence_raw_1_9`
- `sam.arousal_raw_1_9`
- `slider.valence_raw_0_100`
- `slider.arousal_raw_0_100`
- `ekman.rank_order`

The rank-order control intentionally uses move-up/move-down buttons rather than
drag-and-drop so it maps directly to controller, hand ray, and Compose button
input paths.

## Asset Note

SAM SVGs are copied from `cwi-dis/self-assessment-manikins-svg` and retain the
BSD-2-Clause license included at:

```text
assets/sam/LICENSE-BSD-2-Clause.txt
```
