TrainingMod = {
    checkAll(sender) {
        var toggles = sender.getElementsByClassName('keyword-button')

    },
    uncheckAll(sender) {

    },

    goBackHook() {
        // Use this function to check menu settings on exit, return through localhost

        $('.is-focused').addClass('is-pause-anim')
        $('#ret-button').addClass('is-focus')

        disabledOtherLink()

        playSound('cancel')

        fadeOutPage(function() {
            window.history.back()
        })


        var url = "http://localhost/"

        $(".l-grid").each(function() {
            var section = this.id;

            url += section + "?"

            var children = this.children;
            for (var i = 0; i < children.length; i++) {
                var child = children[i];
                if (child.innerHTML.includes("is-appear")) {
                    url += child.getAttribute("val") + ",";
                }
            }

            url += "&"
        });

        location.href = url;
    },


    clickToggle(e) {
        var toggleImage = e.children[0];
        if (toggleImage.innerHTML.includes("is-appear")) {
            toggleImage.innerHTML = toggleImage.innerHTML.replace("is-appear", "is-hidden");
        } else {
            toggleImage.innerHTML = toggleImage.innerHTML.replace("is-hidden", "is-appear");
        }
    },

    initSliders() {
        // todo: loop through
        noUiSlider.create(document.getElementsByName('range_slider')[0], {
            start: {
                // {
                //     value
                // }
            },
            range: {
                'min': {
                    // {
                    //     min
                    // }
                },
                'max': {
                    // {
                    //     max
                    // }
                }
            }
        })
    },

    init() {
        if (isNx) {
            window.nx.footer.setAssign('B', '', goBackHook, {
                se: ''
            })
        }
    }
}

window.onload = function() {
    TrainingMod.init()
}