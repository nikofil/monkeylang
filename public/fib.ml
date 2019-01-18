<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Fibonacci</title>
  </head>
  <body>
    <%
        if (post["a"] != null) {
    %>
    Result is:
    <%
            let f = fn(x) {
                let res = if (x > 1) {
                    f(x-1) + f(x-2)
                } else {
                    1
                }
                return res
            };
            println(f(post["a"]));
        }
    %>
    <form action="#" method="POST">
        <input name="a" />
        <input type="submit" />
    </form>
  </body>
</html>
