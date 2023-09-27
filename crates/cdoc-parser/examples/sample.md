# Gaze estimation

#block{
This is the first mandatory exercise which means you will have to hand in this Jupyter Notebook with your implementation and notes. This exercise is split into multiple parts which have to be submitted together. The submission deadline is available on LearnIT.
}

#block{
#content(type=task, param=title, title="Overview of mandatory tasks", label=Task)
}


## Overview

#block{
#note(class=info, width=40%, float=right){
The location of the *fovea* on the retina varies between people ($\pm$ 5 degrees). Consequently, a gaze model has to be trained (calibrated) for a specific person to be accurate. This difference is shown in #ref(id=kappa).
}

In this exercise you will implement a regression model to estimate where a person is looking (this is known as *gaze*). Gaze estimation is performed by capturing images of a user's eye as shown in #ref(id=model) and mapping them to screen positions using a function $f_\mathbf{w}(x, y)$. Humans look at things by orienting their eyes so that the light from the point of focus hits the *Fovea* (a point on the retina). The Fovea is not located directly behind the pupil, but at a person-specific angle, as shown in #ref(id=kappa). The pupil position can be used to infer gaze, but to obtain accurate gaze estimates requires training data (called calibration).
}block#



#block(width=100%){

#block(float=left, width=50%){

#figure|kappa(
url="material/W05/kappa.jpg",
caption={Shows the distinction between the visual and optical axes. The optical axis is defined as an axis perpendicular to the lens behind the pupil. The visual axis is personally dependent and is determined by the placement of the *fovea*.},
alignment=left,
width=400px)
}

#block(float=right, width=50%){

#figure|model(
url="material/W05/model.png",
caption={Diagram of a gaze estimation system. The eye, which is directed at a specific point on
the screen is captured by the camera. The two red lines represent an unknown transformation from image
to eye and eye to screen. We learn this transformation directly which is shown as $f_{\mathbf{w} }(x, y)$ in the diagram.},
width=400px)
}
}block#



### Gaze mapping function
The goal of this exercise is to estimate the gaze of image sequences using a regression model. Define $f_{\mathbf{w}}(x, y)$ as the gaze  model which maps pupil positions $(x, y)$ to screen coordinates $(x', y')$. The model parameters $\mathbf{w}$ are learned using a set of paired pupil and screen positions.


### About data
We provide a dataset of training and test data. The data has been captured by asking a participant to look at a specific point on a screen while capturing eye images. The pupil shape is detected for each image using ellipse approximation. The ellipses are parametrised by five parameters, `cx, cy, ax, ay, angle`. The center of the ellipse is assumed to be the center of the pupil.


Each image sequence contains 9 images for calibration and a varying number of images for testing. The calibration samples always represent the same 9 screen positions which form a simple 3 by 3 grid. An example of calibration images are shown in #ref(calibration). For each sequence, you will use the 9 calibration samples to train a regression model and then use the model
to predict gaze positions for the rest of the images.


#block(width=100%){
#figure|calibration(
url="material/W05/calibration.jpg",
caption="Calibration images. All image sequences contain 9 calibration images
which all have equivalent gaze positions.",
width=500px)
}

#note(class=info){
The real notebook (the one in the materials repository) contains some extra utility code that has been hidden here for brevity. The code is fully commented and we recommend you read it whenever you are in doubt about what is happening.
}