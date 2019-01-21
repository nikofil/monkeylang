<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Slow page</title>
  </head>
  <body>
    Big decreasing list:
    <%
        let make_list = fn(elements, cur) {
            return if (elements == 0) {
                cur
            } else {
                push(map(make_list(elements-1, cur), fn(x) {x+1}), 0)
            };
        };
        let big_list = make_list(100, []);
        println(big_list);
    %>
  </body>
</html>
