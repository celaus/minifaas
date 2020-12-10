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

async function getTrigger() {
  let trigger = {};
    switch ( $("input[name='fn-trigger-options']:checked").val()) {
      case "http":
        const http_trigger = $("#fn-trigger-select").val();
        trigger = {
          "type": "Http",
          "when": http_trigger.match(HTTP_TRIGGER_PARSER)[1]
        };    
        break;
      case "timer":
        const cron_exp =  $("#fn-trigger-cron").val();
        trigger = {
          "type": "Interval",
          "when": cron_exp.trim()
        };    
        break;
      default:
        trigger = {
          "type": "None",
        }
        break;
    }
    return trigger;
}

async function saveNewFunction() {
  let name = document.getElementById("fn-name").value;
  saveFunction(name);
}

async function saveFunction(name) {
  if (name.trim()) {
    const lang = document.getElementById("fn-lang-select").value;

    let code = editor.getValue();
    const trigger = await getTrigger();
    
    let payload = {
      "id": "",
      "name": name,
      "code": code,
      "trigger": trigger,
      "language": { "lang": lang },
      "timestamp": new Date().toISOString()
    };
    console.log(payload);
    await fetch(API_URL, {
      method: 'put',
      headers: {
        "Content-type": "application/json; charset=UTF-8"
      },
      body: JSON.stringify(payload)
    }).then(async resp => {
      if (resp.ok) {
        $("#alert-ok-text").text(`Couldn't save function: ${ name }`);
        $("#alert-ok").show();
        //window.location = "/?show=" + name;
      } else {
        const response_text = await resp.text();
        $("#alert-ok-text").text(`Couldn't save function: ${ response_text }`);
        $("#alert-ok").show();
      }
    })
  }
  else {
    $("#alert-ok-text").text("NEIN").show();
  }
}

async function removeFunction(name) {
  await fetch(`${API_URL}/${name}`, {
    method: 'delete'
  }).then(_ => location.reload())
}

async function fetchLogs(name) {
  let logs = await fetch(`/api/v1/logs/${name}/0/1000?format=html`);
  $("#fn-logs").html(await logs.text());
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


$(document).ready(function() { 
  setInterval(async () => {
    if ($("#fn-name").val()) {
      const name = $("#fn-name").val();
      await fetchLogs(name);
    }
  }, 1000)
 });