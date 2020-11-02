const API_URL = "/api/v1/f";
const HTTP_TRIGGER_PARSER = /HTTP \(([A-Z]+)\)/;

const editor = CodeMirror.fromTextArea(document.getElementById("editor"), {
  lineNumbers: true,
  mode: "text/javascript",
  matchBrackets: true,
  lineWrapping: true,
});

async function selectToShow(name, code, trigger) {
  document.getElementById("fn-name").value = name;

  const options = Array.apply(null, document.getElementById("fn-trigger-select").options);
  const selected_trigger = options.find(v => v.value == trigger);

  if (selected_trigger !== undefined)
    selected_trigger.selected = true;
  editor.setValue(code);
  editor.focus();
  return true;
}

async function saveFunction() {
  let name = document.getElementById("fn-name").value;
  if (name.trim()) {
    const http_trigger = document.getElementById("fn-trigger-select").value;
    const lang = document.getElementById("fn-lang-select").value;

    console.log(http_trigger);
    let code = editor.getValue();
    let payload = {
      "id": name,
      "name": name,
      "code": code,
      "trigger": {
        "type": "Http",
        "when": http_trigger.match(HTTP_TRIGGER_PARSER)[1]
      },
      "language": { "lang": lang },
      "timestamp": new Date().toISOString()
    };
    await fetch(API_URL, {
      method: 'put',
      headers: {
        "Content-type": "application/json; charset=UTF-8"
      },
      body: JSON.stringify(payload)
    }).then(resp => {
      if (resp.ok) {
        $("#alert-ok-text").html("Saved!").show();
      } else {
        $("#alert-ok-text").html("bad news").show();
      }
    })
  }
  else {
    $("#alert-ok-text").html("NEIN").show();
  }
}

async function removeFunction(name) {
  await fetch(`${API_URL}/${name}`, {
    method: 'delete'
  }).then(_ => location.reload())
}

async function callFunction(name) {
  let payload = {
    "id": name,
    "name": name,
    "code": "code",
    "trigger": "Http",
    "method": "GET",
    "language": { "lang": "JavaScript" },
    "timestamp": new Date().toISOString()
  };
  console.log(payload);
  console.log(await fetch(`/f/call/${name}`, {
    method: 'POST',
    headers: {
      "Content-type": "application/json; charset=UTF-8"
    },
    body: JSON.stringify(payload)
  }));
}