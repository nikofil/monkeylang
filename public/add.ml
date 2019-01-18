<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <title>Addition</title>
  </head>
  <body>
    <%
        if (post["a"] != null) {
    %>
    Result is:
    <%
            println(post["a"] + post["b"])
        }
    %>
    <form action="#" method="POST">
        <input name="a" />
        <input name="b" />
        <input type="submit" />
    </form>
  </body>
</html>
