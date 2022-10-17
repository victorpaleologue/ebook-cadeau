<?php

namespace App\Controller;

use Symfony\Bundle\FrameworkBundle\Controller\AbstractController;
use Symfony\Component\HttpFoundation\Response;
use Symfony\Component\Routing\Annotation\Route;

class ChoiceController extends AbstractController
{
    #[Route('/choice', name: 'app_choice')]
    public function index(): Response
    {
        if(isset ($_POST['epub']))
        {   
        $epub=htmlentities($_POST['epub']);
        return $this->render('choice/index.html.twig', [
            'epub' => $epub,
        ]);
        }

        // return $this->render('choice/index.html.twig', [
        //     'controller_name' => 'ChoiceController',
        // ]);
    }
}
