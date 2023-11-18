<figure class="figure" {% if id %}id="{{ id }}"{% endif %}>
<img src="data:image/png;base64,{{ url | embed }}" class="figure-img img-fluid rounded" style="width:{{width | default(value='50%')}}" />
{% if caption %}<figcaption class="figure-caption has-text-{{ alignment | default(value='centered') }}">Figure {{ num }}: {{caption | safe}}</figcaption>{% endif %}
</figure>