"use strict";

const PANEL_ID = "emotion_induction_sam_preview";
const SCHEMA_VERSION = 1;
const QUEST_PANEL_FRAME = { width_dp: 1080, height_dp: 720 };

const CONDITIONS = [
  { id: "induction_a", label: "Induction A" },
  { id: "induction_b", label: "Induction B" },
  { id: "induction_c", label: "Induction C" },
  { id: "induction_d", label: "Induction D" }
];

const COUNTERBALANCE_ORDERS = [
  { id: "order_01", label: "Order 01: A B D C", condition_ids: ["induction_a", "induction_b", "induction_d", "induction_c"] },
  { id: "order_02", label: "Order 02: B C A D", condition_ids: ["induction_b", "induction_c", "induction_a", "induction_d"] },
  { id: "order_03", label: "Order 03: C D B A", condition_ids: ["induction_c", "induction_d", "induction_b", "induction_a"] },
  { id: "order_04", label: "Order 04: D A C B", condition_ids: ["induction_d", "induction_a", "induction_c", "induction_b"] }
];

const EKMAN_EMOTIONS = [
  { id: "happiness", label: "Happiness" },
  { id: "surprise", label: "Surprise" },
  { id: "fear", label: "Fear" },
  { id: "anger", label: "Anger" },
  { id: "disgust", label: "Disgust" },
  { id: "sadness", label: "Sadness" }
];

const CONTROL_MODEL = [
  {
    id: "counterbalance.order_id",
    label: "Counterbalance order",
    type: "select",
    default: "order_01",
    options: COUNTERBALANCE_ORDERS.map((order) => order.id),
    editable: "preview-only",
    validation: "must be one of counterbalance order ids",
    native_state_field: "questionnaire_state.counterbalance_order_id"
  },
  {
    id: "condition.active_index",
    label: "Active condition index",
    type: "segmented-preview-navigation",
    default: 1,
    min: 1,
    max: 4,
    step: 1,
    editable: "preview-only",
    validation: "must be 1..4",
    native_state_field: "questionnaire_state.condition_index"
  },
  {
    id: "sam.valence_raw_1_9",
    label: "SAM valence",
    type: "pictographic-choice",
    default: 5,
    min: 1,
    max: 9,
    step: 1,
    options: [1, 2, 3, 4, 5, 6, 7, 8, 9],
    editable: "editable",
    validation: "required integer 1..9",
    result_json_field: "answers.emotion_assessment.sam.valence_raw_1_9"
  },
  {
    id: "sam.arousal_raw_1_9",
    label: "SAM arousal",
    type: "pictographic-choice",
    default: 5,
    min: 1,
    max: 9,
    step: 1,
    options: [1, 2, 3, 4, 5, 6, 7, 8, 9],
    editable: "editable",
    validation: "required integer 1..9",
    result_json_field: "answers.emotion_assessment.sam.arousal_raw_1_9"
  },
  {
    id: "slider.valence_raw_0_100",
    label: "Valence slider",
    type: "range",
    default: 50,
    min: 0,
    max: 100,
    step: 1,
    editable: "editable",
    validation: "required integer 0..100",
    result_json_field: "answers.emotion_assessment.sliders.valence_raw_0_100"
  },
  {
    id: "slider.arousal_raw_0_100",
    label: "Arousal slider",
    type: "range",
    default: 50,
    min: 0,
    max: 100,
    step: 1,
    editable: "editable",
    validation: "required integer 0..100",
    result_json_field: "answers.emotion_assessment.sliders.arousal_raw_0_100"
  },
  {
    id: "ekman.rank_order",
    label: "Ekman emotion rank order",
    type: "rank-order",
    default: EKMAN_EMOTIONS.map((emotion) => emotion.id),
    options: EKMAN_EMOTIONS.map((emotion) => emotion.id),
    editable: "editable",
    validation: "all listed emotion ids exactly once",
    result_json_field: "answers.emotion_assessment.ekman_rank_order"
  }
];

function defaultAssessment() {
  return {
    sam: {
      valence_raw_1_9: 5,
      arousal_raw_1_9: 5
    },
    sliders: {
      valence_raw_0_100: 50,
      arousal_raw_0_100: 50
    },
    ekman_rank_order: EKMAN_EMOTIONS.map((emotion) => emotion.id),
    complete: false
  };
}

function makeState() {
  return {
    panel_id: PANEL_ID,
    schema_version: SCHEMA_VERSION,
    counterbalance_order_id: "order_01",
    active_condition_position: 1,
    responses_by_condition: CONDITIONS.map((condition, index) => ({
      condition_id: condition.id,
      preview_condition_label: condition.label,
      assigned_position: index + 1,
      assessment: defaultAssessment()
    }))
  };
}

function makeEdgeState() {
  const state = makeState();
  state.counterbalance_order_id = "order_04";
  state.active_condition_position = 4;
  state.responses_by_condition.forEach((entry, index) => {
    entry.assessment.sam.valence_raw_1_9 = index === 3 ? 1 : 9;
    entry.assessment.sam.arousal_raw_1_9 = index === 3 ? 9 : 1;
    entry.assessment.sliders.valence_raw_0_100 = index === 3 ? 0 : 100;
    entry.assessment.sliders.arousal_raw_0_100 = index === 3 ? 100 : 0;
    entry.assessment.ekman_rank_order = EKMAN_EMOTIONS.map((emotion) => emotion.id).reverse();
    entry.assessment.complete = index < 3;
  });
  return state;
}

let state = makeState();

const elements = {
  conditionStatus: document.getElementById("conditionStatus"),
  conditionLabel: document.getElementById("conditionLabel"),
  samRows: document.getElementById("samRows"),
  samCompletion: document.getElementById("samCompletion"),
  sliderRows: document.getElementById("sliderRows"),
  rankList: document.getElementById("rankList"),
  rankStatus: document.getElementById("rankStatus"),
  validationSummary: document.getElementById("validationSummary"),
  completeCondition: document.getElementById("completeCondition"),
  orderSelect: document.getElementById("orderSelect"),
  conditionButtons: document.getElementById("conditionButtons"),
  loadDefault: document.getElementById("loadDefault"),
  loadEdge: document.getElementById("loadEdge"),
  exportState: document.getElementById("exportState"),
  jsonOutput: document.getElementById("jsonOutput")
};

function activeOrder() {
  return COUNTERBALANCE_ORDERS.find((order) => order.id === state.counterbalance_order_id) || COUNTERBALANCE_ORDERS[0];
}

function activeConditionId() {
  const order = activeOrder();
  return order.condition_ids[state.active_condition_position - 1] || order.condition_ids[0];
}

function conditionLabelFor(id) {
  return (CONDITIONS.find((condition) => condition.id === id) || CONDITIONS[0]).label;
}

function responseFor(conditionId = activeConditionId()) {
  let response = state.responses_by_condition.find((entry) => entry.condition_id === conditionId);
  if (!response) {
    response = {
      condition_id: conditionId,
      preview_condition_label: conditionLabelFor(conditionId),
      assigned_position: state.active_condition_position,
      assessment: defaultAssessment()
    };
    state.responses_by_condition.push(response);
  }
  return response;
}

function activeAssessment() {
  return responseFor().assessment;
}

function setState(nextState) {
  state = JSON.parse(JSON.stringify(nextState));
  render();
}

function render() {
  renderOrderSelect();
  renderConditionButtons();
  renderHeader();
  renderSamRows();
  renderSliders();
  renderRankList();
  renderValidation();
  renderExport();
}

function renderOrderSelect() {
  elements.orderSelect.replaceChildren();
  COUNTERBALANCE_ORDERS.forEach((order) => {
    const option = document.createElement("option");
    option.value = order.id;
    option.textContent = order.label;
    option.selected = order.id === state.counterbalance_order_id;
    elements.orderSelect.appendChild(option);
  });
}

function renderConditionButtons() {
  elements.conditionButtons.replaceChildren();
  const order = activeOrder();
  order.condition_ids.forEach((conditionId, index) => {
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = `${index + 1}. ${conditionLabelFor(conditionId).replace("Induction ", "")}`;
    button.setAttribute("aria-pressed", String(index + 1 === state.active_condition_position));
    button.addEventListener("click", () => {
      state.active_condition_position = index + 1;
      render();
    });
    elements.conditionButtons.appendChild(button);
  });
}

function renderHeader() {
  const conditionId = activeConditionId();
  const response = responseFor(conditionId);
  response.assigned_position = state.active_condition_position;
  response.preview_condition_label = conditionLabelFor(conditionId);
  elements.conditionStatus.textContent = `Condition ${state.active_condition_position} of 4`;
  elements.conditionLabel.textContent = conditionLabelFor(conditionId);
}

function renderSamRows() {
  const assessment = activeAssessment();
  const rows = [
    {
      id: "valence",
      label: "Valence",
      low: "Unpleasant",
      high: "Pleasant",
      field: "valence_raw_1_9"
    },
    {
      id: "arousal",
      label: "Arousal",
      low: "Calm",
      high: "Excited",
      field: "arousal_raw_1_9"
    }
  ];
  elements.samRows.replaceChildren();
  rows.forEach((row) => {
    const container = document.createElement("div");
    container.className = "sam-row";

    const label = document.createElement("div");
    label.className = "row-label";
    label.innerHTML = `<strong>${row.label}</strong><span>${row.low}</span><span>${row.high}</span>`;
    container.appendChild(label);

    const options = document.createElement("div");
    options.className = "sam-options";
    for (let score = 1; score <= 9; score += 1) {
      const button = document.createElement("button");
      button.type = "button";
      button.className = "sam-choice";
      button.id = `sam.${row.id}.${score}`;
      button.setAttribute("aria-label", `${row.label} ${score}`);
      button.setAttribute("aria-pressed", String(assessment.sam[row.field] === score));

      const img = document.createElement("img");
      img.src = `assets/sam/${row.id}/${row.id}-${score}.svg`;
      img.alt = "";
      img.draggable = false;

      const number = document.createElement("span");
      number.textContent = String(score);

      button.appendChild(img);
      button.appendChild(number);
      button.addEventListener("click", () => {
        assessment.sam[row.field] = score;
        assessment.complete = false;
        render();
      });
      options.appendChild(button);
    }
    container.appendChild(options);
    elements.samRows.appendChild(container);
  });
}

function renderSliders() {
  const assessment = activeAssessment();
  const sliders = [
    {
      id: "slider.valence_raw_0_100",
      label: "Valence",
      low: "Unpleasant",
      high: "Pleasant",
      field: "valence_raw_0_100"
    },
    {
      id: "slider.arousal_raw_0_100",
      label: "Arousal",
      low: "Calm",
      high: "Excited",
      field: "arousal_raw_0_100"
    }
  ];
  elements.sliderRows.replaceChildren();
  sliders.forEach((slider) => {
    const row = document.createElement("div");
    row.className = "slider-row";
    row.innerHTML = `
      <header>
        <strong>${slider.label}</strong>
        <span class="slider-value" id="${slider.id}.value">${assessment.sliders[slider.field]}</span>
      </header>
      <input id="${slider.id}" type="range" min="0" max="100" step="1" value="${assessment.sliders[slider.field]}">
      <div class="slider-axis"><span>${slider.low}</span><span>${slider.high}</span></div>
    `;
    const input = row.querySelector("input");
    input.addEventListener("input", () => {
      assessment.sliders[slider.field] = Number(input.value);
      assessment.complete = false;
      render();
    });
    elements.sliderRows.appendChild(row);
  });
}

function renderRankList() {
  const assessment = activeAssessment();
  const order = assessment.ekman_rank_order.filter((id) => EKMAN_EMOTIONS.some((emotion) => emotion.id === id));
  EKMAN_EMOTIONS.forEach((emotion) => {
    if (!order.includes(emotion.id)) {
      order.push(emotion.id);
    }
  });
  assessment.ekman_rank_order = order;
  elements.rankList.replaceChildren();
  order.forEach((emotionId, index) => {
    const emotion = EKMAN_EMOTIONS.find((item) => item.id === emotionId);
    const item = document.createElement("li");
    item.className = "rank-item";
    item.innerHTML = `
      <span class="rank-number">${index + 1}</span>
      <span class="rank-label">${emotion.label}</span>
      <button class="rank-move" type="button" aria-label="Move ${emotion.label} up" ${index === 0 ? "disabled" : ""}>^</button>
      <button class="rank-move" type="button" aria-label="Move ${emotion.label} down" ${index === order.length - 1 ? "disabled" : ""}>v</button>
    `;
    const buttons = item.querySelectorAll("button");
    buttons[0].addEventListener("click", () => moveRank(index, -1));
    buttons[1].addEventListener("click", () => moveRank(index, 1));
    elements.rankList.appendChild(item);
  });
  elements.rankStatus.textContent = `${order.length} ranked`;
}

function moveRank(index, delta) {
  const assessment = activeAssessment();
  const next = index + delta;
  if (next < 0 || next >= assessment.ekman_rank_order.length) {
    return;
  }
  const copy = assessment.ekman_rank_order.slice();
  const [item] = copy.splice(index, 1);
  copy.splice(next, 0, item);
  assessment.ekman_rank_order = copy;
  assessment.complete = false;
  render();
}

function validationErrors(assessment = activeAssessment()) {
  const errors = [];
  if (!Number.isInteger(assessment.sam.valence_raw_1_9) || assessment.sam.valence_raw_1_9 < 1 || assessment.sam.valence_raw_1_9 > 9) {
    errors.push("Select SAM valence.");
  }
  if (!Number.isInteger(assessment.sam.arousal_raw_1_9) || assessment.sam.arousal_raw_1_9 < 1 || assessment.sam.arousal_raw_1_9 > 9) {
    errors.push("Select SAM arousal.");
  }
  ["valence_raw_0_100", "arousal_raw_0_100"].forEach((field) => {
    if (!Number.isInteger(assessment.sliders[field]) || assessment.sliders[field] < 0 || assessment.sliders[field] > 100) {
      errors.push(`Slider ${field} must be 0..100.`);
    }
  });
  const unique = new Set(assessment.ekman_rank_order);
  if (unique.size !== EKMAN_EMOTIONS.length || assessment.ekman_rank_order.length !== EKMAN_EMOTIONS.length) {
    errors.push("Rank each Ekman emotion exactly once.");
  }
  return errors;
}

function renderValidation() {
  const errors = validationErrors();
  elements.validationSummary.classList.toggle("error", errors.length > 0);
  elements.validationSummary.textContent = errors.length === 0 ? "Complete" : errors[0];
  elements.completeCondition.disabled = errors.length > 0;
  const assessment = activeAssessment();
  elements.completeCondition.textContent = assessment.complete ? "Condition marked complete" : "Mark condition complete";
}

function exportObject() {
  const order = activeOrder();
  return {
    panel_id: PANEL_ID,
    schema_version: SCHEMA_VERSION,
    quest_panel_frame: QUEST_PANEL_FRAME,
    native_contract_authority: {
      protocol_version: "quest.questionnaire.v1",
      schema_id: "emotion-induction-sam-v1",
      open_stage: "emotion_assessment",
      screen_sequence: ["emotion_assessment"],
      result_owner: "caller-owned content URI"
    },
    preview_transfer_note: "Browser preview state is a layout and fixture artifact only. Native Android/Compose request parsing, result writing, focus, and headset validation remain authoritative.",
    control_model: CONTROL_MODEL,
    pages: [
      {
        id: "emotion_assessment",
        groups: [
          { id: "sam", fields: ["sam.valence_raw_1_9", "sam.arousal_raw_1_9"] },
          { id: "sliders", fields: ["slider.valence_raw_0_100", "slider.arousal_raw_0_100"] },
          { id: "ekman_rank", fields: ["ekman.rank_order"] }
        ]
      }
    ],
    counterbalance: {
      order_id: order.id,
      condition_ids: order.condition_ids,
      editable_in_preview: true,
      editable_in_native_panel: false
    },
    active_condition_position: state.active_condition_position,
    active_condition_id: activeConditionId(),
    responses_by_condition: order.condition_ids.map((conditionId, index) => {
      const response = responseFor(conditionId);
      return {
        condition_position: index + 1,
        condition_id: conditionId,
        assessment: response.assessment
      };
    })
  };
}

function renderExport() {
  elements.jsonOutput.textContent = JSON.stringify(exportObject(), null, 2);
}

elements.orderSelect.addEventListener("change", () => {
  state.counterbalance_order_id = elements.orderSelect.value;
  state.active_condition_position = 1;
  render();
});

elements.completeCondition.addEventListener("click", () => {
  const assessment = activeAssessment();
  if (validationErrors(assessment).length === 0) {
    assessment.complete = true;
    render();
  }
});

elements.loadDefault.addEventListener("click", () => setState(makeState()));
elements.loadEdge.addEventListener("click", () => setState(makeEdgeState()));
elements.exportState.addEventListener("click", async () => {
  const text = JSON.stringify(exportObject(), null, 2);
  elements.jsonOutput.textContent = text;
  if (navigator.clipboard && window.isSecureContext) {
    try {
      await navigator.clipboard.writeText(text);
    } catch (_) {
      // Clipboard support varies for file:// previews; the JSON remains visible.
    }
  }
});

render();
