const invoke = window.__TAURI__.core.invoke;
const events = window.__TAURI__.event;

// Reading pressed code
events.listen('pressed-code', ({ payload }) => {
    const { code } = payload;
    
    console.log(`Pressed code '${code}'`);
    
    let pressed_code = document.querySelector("#pressed-code");
    pressed_code.value = code;
});

// Update binds list
function update_binds() {
    invoke("get_binds", {})
        .then(binds => {
            let binds_container = document.querySelector("#main .binds");
            
            binds.forEach((bind) => {
                binds_container.insertAdjacentHTML('beforeend', bind)
            });
        })
        .catch(e => console.error(e))
}

const form_timers = new Map();

document.addEventListener('DOMContentLoaded', () => {
    // Get binds list:
    update_binds();

    // Pressed button handler:
    document.querySelector("#header #pressed-code").addEventListener("click", (e) => {
        navigator.clipboard.writeText(e.target.value)
            .then(() => {})
            .catch(err => {
                console.error('Failed to copy code: ', err);
            });
    });

    // Add a new bind:
    document.querySelector("#main button.add-bind").addEventListener("click", () => {
        invoke("add_bind", {})
            .then(bind => {
                let binds_container = document.querySelector("#main .binds");
                binds_container.insertAdjacentHTML("beforeend", bind)
            })
            .catch(e => console.error(e))
    });

    // Update bind data:
    document.querySelector("#main .binds").addEventListener("input", (ev) => {
        let target = ev.target;
        let form = new Form(target.closest("form"));
        let data = form.serialize();

        // reset form timer:
        if (form_timers.has(data.id)) {
            clearTimeout(form_timers.get(data.id));
        }

        // start form timer:
        form_timers.set(data.id, setTimeout(async () => {
            if (data.action === "KeyboardPress") {
                let keys = [];

                if (data.value !== "") {
                    data.value.split(",").forEach((key) => {
                        if (key.trim() !== '') {
                            keys.push(key.trim());
                        }
                    });
                }
                // console.log(keys);
                
                let action = {};
                action[data.action] = keys;
                data.action = action;
            }
            else if (data.action === "BrowserOpen") {
                let action = {};
                action[data.action] = data.value;
                data.action = action;
            }
            data.value = undefined;
            
            invoke("update_bind", { data })
                .then((bind_html) => {
                    form_timers.delete(data.id);

                    let bind = document.querySelector(`#main .binds .bind[bind-id="${data.id}"]`);
                    bind.outerHTML = bind_html;
                })
                .catch(e => console.error(e));
        }, 1000));
    });

    // Remove bind:
    document.querySelector("#main .binds").addEventListener("click", (event) => {
        let target = event.target;
        
        if (target.classList.contains("remove")) {
            let id = target.getAttribute("target");
            let bind = document.querySelector(`#main .binds .bind[bind-id="${id}"]`);

            invoke("remove_bind", { id })
                .then(() => {
                    bind.remove();
                })
                .catch(e => console.error(e));
        }
    });
});


// Form controller
class Form {
    constructor(form) {
        // selector:
        if (typeof form === "string") {
            this.form = document.querySelector(form);
            if (!this.form) throw new Error("Form not found");
        }
        // element:
        else if (form instanceof Element) {
            this.form = form;
        }
        // error:
        else {
            throw new Error("Form constructor expects a selector string or a DOM element");
        }
    }

    // Get field by name
    field(name) {
        let fields = this.form.querySelectorAll(`input[name="${name}"]`);
        if (!fields.length) return undefined;

        let type = fields[0].type;

        // radio:
        if (type === "radio") {
            let checked = this.form.querySelector(`input[name="${name}"]:checked`);
            return checked ? checked.value : null;
        }
        // checkbox:
        else if (type === "checkbox") {
            if (fields.length > 1) {
                return Array.from(fields)
                    .filter(f => f.checked)
                    .map(f => f.value);
            }
            return fields[0].checked;
        }
        // other:
        else {
            return fields[0].value;
        }
    }

    // Serialize form to json
    serialize() {
        let data = {};

        this.form.querySelectorAll("input[name]").forEach(el => {
            let name = el.name;
            if (data.hasOwnProperty(name)) return;

            if (el.type === "checkbox") {
                data[el.name] = el.checked;
            }
            else if (el.type === "radio") {
                if (el.checked) {
                    data[el.name] = el.value;
                }
            }
            else {
                data[el.name] = el.value;
            }
        });

        return data;
    }
}
