// RELEASE
console.log = function(){}

window['ulpath'] = 1;

function locate_elem(txt1, txt2, rootnode) {
    var nroute1 = [];
    var nroute2 = [];
    
    function check_inner_text(ele, rou) {
	let nchild = ele.childNodes.length;
	for (let i=0; i < nchild; i++) {
	    let c = ele.childNodes[i];
	    if (c.innerText == txt1) {
		if (nroute1.length == 0)
		    nroute1 = [...rou, i];
	    } else if (c.innerText == txt2) {
		if (nroute2.length == 0)
		    nroute2 = [...rou, i];
	    } else {
		check_inner_text(c, [...rou, i]);
	    }
	}
    }
    check_inner_text(rootnode, []);


    if (nroute1.length != nroute2.length) {
	return null;
    }

    console.log(nroute1, nroute2);
    
    let fork_idx = -1;
    for (let i = 0; i < nroute1.length; i++) {
	if (nroute1[i] != nroute2[i]) {
	    fork_idx = i;
	    break;
	}
    }

    let upper_idx = [];
    let lower_idx = [];
    let upper_path = 'document.body'; // debug convenience
    let lower_path = ''; // debug convenience

    if (fork_idx == -1) 
	for (let i = 0; i < nroute1.length; i++) {
	    let u_idx = nroute1[i];
	    upper_idx.push(u_idx);
	    upper_path += ('.childNodes[' + u_idx + ']');
	}
    else
	for (let i = 0; i < nroute1.length; i++) {
	    let u_idx = nroute1[i];
	    if (i < fork_idx) {
		upper_idx.push(u_idx);
		upper_path += ('.childNodes[' + u_idx + ']');
	    } else if (i > fork_idx) {
		lower_idx.push(u_idx);
		// lower_path += ('.childNodes[' + u_idx + ']');
	    }
	}

    window['ulpath'] = {
	upper_idx,
	lower_idx,
    	upper_path,
    	// lower_path,
    }

    return window['ulpath'];
}

// DEBUG
// function send_back(send_what) { console.log("here is~~~", send_what); }
// RELEASE
var send_back = arguments[arguments.length - 1]; 

clearInterval(window['twtl-get-ulpath']); // kill existing timer
window['twtl-get-ulpath'] = setInterval(function() {
    if (document.body.innerText.includes("__________0__________")) {
	var result = locate_elem(
	    '__________1__________',
	    '__________0__________',
	    document.body);
	send_back(result);
	clearInterval(window['twtl-get-ulpath']);
    }
} ,1000);

