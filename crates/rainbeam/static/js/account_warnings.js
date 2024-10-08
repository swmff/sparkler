(() => {
    const self = reg_ns("account_warnings");

    self.define("delete", function (_, id) {
        if (!confirm("Are you sure you want to do this?")) {
            return;
        }

        fetch(`/api/auth/warnings/${id}`, {
            method: "DELETE",
        })
            .then((res) => res.json())
            .then((res) => {
                trigger("app:shout", [
                    res.success ? "tip" : "caution",
                    res.message || "Warning deleted!",
                ]);

                document
                    .getElementById(`warning:${id}`)
                    .setAttribute("disabled", "fully");
            });
    });
})();
