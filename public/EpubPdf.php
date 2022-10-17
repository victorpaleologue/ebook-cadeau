<!DOCTYPE html>
<html>

<head>
    <title>Page de traitement</title>
    <meta charset="utf-8">
</head>

<body>

    <p>Voici le QR code a scanner pour acceder a l'URL</p> <br>

    <?php

    $epub = htmlentities($_POST['epub']);

    if (isset($_POST['epub'])) {
        echo 'Voici le QR code a scanner pour acceder a l\'URL <img src="https://api.qrserver.com/v1/create-qr-code/?data=' . $epub . '" alt="" title="" />';
    }
    ?>
</body>

</html>