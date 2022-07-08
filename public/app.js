const form = document.querySelector('#weha');

form.addEventListener('submit', async function (event) {
  event.preventDefault();
  const formData = new FormData(event.target);
  const formProps = Object.fromEntries(formData);

  try {
    const response = await fetch('/login', {
      method: 'POST',
      redirect: 'follow',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(formProps),
    });
    if (response.ok) {
      return (window.location.href = '/authenticated');
    }

    // handle is status is not ok
  } catch (error) {
    // handle error
  }
});
