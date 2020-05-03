const API_URL = "/api/v1/f";
const editor = CodeMirror.fromTextArea(document.getElementById("editor"), {
  lineNumbers: true,
  mode: "text/javascript",
  matchBrackets: true,
  lineWrapping: true,
});

async function selectToShow(name) {
  document.getElementById("fn-name").value = name;
  let response = await (await fetch(`/f/impl/${name}`)).json();
  let code = response.code;
  let trigger = "Http trigger (GET)";

  const options = Array.apply(null, document.getElementById("fn-trigger-select").options);
  const selected_trigger = options.find(v => v.value == trigger);

  if (selected_trigger !== undefined)
    selected_trigger.selected = true;
  editor.setValue(code);
  editor.focus();
  return true;
}

function saveFunction() {
  let name = document.getElementById("fn-name").value;
  let code = editor.getValue();
  let payload = {
    "id": name,
    "name": name,
    "code": code,
    "trigger": "Http",
    "method": "GET",
    "lang": "JavaScript",
    "timestamp": new Date().toISOString()
  };
  console.log(payload);
  fetch(API_URL, {
    method: 'put',
    headers: {
      "Content-type": "application/json; charset=UTF-8"
    },
    body: JSON.stringify(payload)
  }).then(_ => location.reload())
}

async function callFunction(name) {
  let response = await fetch(`/f/call/${name}`);
}