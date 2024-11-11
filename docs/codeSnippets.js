/**
 * Removes the initial prompt characters "$ " from the beginning of each line in a multiline string, if present.
 *
 * @param {string} text - The input string, potentially containing multiple lines with prompts at the beginning.
 * @returns {string} - The modified string with initial prompts removed from each line, or the original lines if no prompt is present.
 */
function removePrompt(text) {
    return text.split('\n').map(line => {
        if (line.startsWith("$ ")) {
            return line.slice(2);
        }
        return line;
    }).join('\n');
}


// Overwrite the default `playground_text` function which is used to extract the code from a playground.
function playground_text(playground, hidden = true) {
    let code_block = playground.querySelector("code");
    
    if (window.ace && code_block.classList.contains("editable")) {
        let editor = window.ace.edit(code_block);
        return removePrompt(editor.getValue());
    } else if (hidden) {
        return removePrompt(code_block.textContent);
    } else {
        return removePrompt(code_block.innerText);
    }
}
