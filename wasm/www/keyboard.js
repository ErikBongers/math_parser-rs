var Calc = {
    addButton: function(label, color) {
        let button = document.createElement('div');
        button.id = 'todo';
        button.className = 'grid-item ' + color;
        button.innerHTML = label;
        button.setAttribute('onclick','Calc.typeIt(event)');

        let calcCont = document.querySelector('#calculator');
        calcCont.appendChild(button);
        },
    addButtons: function()
        {
        Calc.addButton("+", "medium");
        Calc.addButton("-", "medium");
        Calc.addButton("*", "medium");
        Calc.addButton("/", "medium");
        Calc.addButton("^", "medium");
        Calc.addButton("!", "medium");
        Calc.addButton("(", "light");
        Calc.addButton(")", "light");
        Calc.addButton("=", "light");
        Calc.addButton(";", "medium");
        },
    typeIt: function(event){
        let theText = event.srcElement.innerHTML;
        let transactionSpec = cm.editor.state.replaceSelection(theText);
        let transaction = cm.editor.state.update(transactionSpec);
        cm.editor.update([transaction]);
        cm.editor.focus();
    } 
};

Calc.addButtons();

