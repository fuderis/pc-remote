use crate::{ prelude::*, Bind, Action };

/// Generates bind HTML code
pub fn generate_bind(id: &str, bind: &Bind) -> Result<String> {
    let code = &bind.code;
    
    let actions = Action::get_all().iter()
        .map(|action| {
            let name = action.to_string();
            let checked = if name == bind.action.to_string() {"checked"}else{""};
            
            fmt!(r##"<option2>
                <input id="option-action-select-{id}-{name}" name="action" value="{name}" type="radio" {checked} style="display:none">
                <label for="option-action-select-{id}-{name}">{name}</label>
            </option2>"##)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let (value, value_disabled) = match &bind.action {
        Action::KeyboardPress(keys) => {
            if !keys.is_empty() {
                let json = serde_json::to_string(&keys)?;
                (json[1..json.len()-2].to_owned(), "")
            } else {
                (str!(), "")
            }
        }

        Action::BrowserOpen(url) => {
            (url.to_owned(), "")
        }

        _ => (str!(), "disabled"),
    };
    let value = value.replace("\"", "").replace(",", ", ");
    let checked = if bind.repeat {"checked"}else{""};

    let bind_html = fmt!(r##"
        <div class="bind" bind-id="{id}">
            <form class="options">
                <input name="id" type="text" value="{id}" style="display:none">

                <div class="option code">
                    <label for="option-code-{id}" class="title">Code:</label>
                    <input id="option-code-{id}" name="code" type="text" value="{code}">
                </div>
                
                <div class="option action">
                    <label for="option-action-{id}" class="title">Action:</label>
                    <select2 id="option-action-{id}">
                        <container>
                            {actions}
                        </container>
                    </select2>
                </div>

                <div class="option value" {value_disabled}>
                    <label for="option-value-{id}" class="title">Value:</label>
                    <input id="option-value-{id}" name="value" type="text" value="{value}">
                </div>

                <div class="option checkbox">
                    <input id="option-repeat-{id}" name="repeat" type="checkbox" {checked} style="display:none">
                    <label for="option-repeat-{id}" class="title">Repeat:</label>
                    <label for="option-repeat-{id}" class="checkbox"></label>
                </div>

                <button class="remove" type="button" target="{id}">
                    <img src="/assets/images/icons/cross-icon.svg" alt="">
                </button>
            </form>
        </div>
    "##);

    Ok(bind_html)
}
