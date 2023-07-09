// RELEASE
console.log = function() {}

function locate_elem(given_txt) {
    var upaths = [];
    function check_inner_text(ele, txt, rou) {
	let nchild = ele.childNodes.length;
	for (let i=0; i < nchild; i++) {
	    let c = ele.childNodes[i];
	    if (c.innerText && c.innerText == (txt)) {
		console.log("yes",c);
		upaths = [...rou, i];  // for one upath
		// upaths.push([...rou, i]); // for many upaths
	    } else {
		check_inner_text(c, txt, [...rou, i]);
	    }
	}
    }
    let back_to_normal = given_txt;
    check_inner_text(document.body, back_to_normal, []); 

    console.log(upaths);
    
    let elems = upaths.map(() => document.body);
    console.log(elems);
    for (let i = 0; i < upaths.length; i++) {
	for (let j = 0; j < upaths[i].length; j++) {
	    // elems[i] is always non-undefined
	    elems[i] = elems[i].childNodes[upaths[i][j]];
	}
    }

    return upaths;
}

// DEBUG
// locate_elem("used when meeting or greeting someone:");

// RELEASE
// DONT FORGET: return  
locate_elem(arguments[0]);
